use itertools::Itertools as _;
use ratatui::layout::Margin;
use ratatui::widgets::Paragraph;
use ratatui::{
    crossterm::event::{self, Event, KeyCode},
    widgets::Block,
};
use ratatui::{
    layout::{Constraint, Layout},
    Frame,
};
use serde_json::{from_str, Value};
use shared::player::{Direction, Player};
use shared::{Map, GFX_PORT};
use std::collections::HashMap;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::net::TcpStream;
use tokio::sync::mpsc;
use tokio::time::Duration;

pub const NORTH_EMOJI: &'static str = "⬆️";
pub const SOUTH_EMOJI: &'static str = "⬇️";
pub const EAST_EMOJI: &'static str = "➡️";
pub const WEST_EMOJI: &'static str = "⬅️";

fn direction_to_emoji(direction: &Direction) -> &'static str {
    match direction {
        Direction::North => NORTH_EMOJI,
        Direction::South => SOUTH_EMOJI,
        Direction::East => EAST_EMOJI,
        Direction::West => WEST_EMOJI,
    }
}

fn draw(frame: &mut Frame, map: &mut Option<Map>, players: &mut Option<HashMap<u16, Player>>) {
    if let (Some(data), Some(players)) = (map, players) {
        let area = frame.area().inner(Margin {
            vertical: 1,
            horizontal: 1,
        });

        let rows =
            Layout::vertical(vec![Constraint::Ratio(1, data.width as u32); data.width]).split(area);

        let mut cols = rows.iter().flat_map(|row| {
            Layout::horizontal(vec![Constraint::Ratio(1, data.height as u32); data.height])
                .split(*row)
                .to_vec()
        });

        for y in 0..data.height {
            for x in 0..data.width {
                let col = cols.next().unwrap();
                let cell = &data.map[y][x];
                let mapped_resources = cell
                    .resources
                    .iter()
                    .map(|(k, &v)| (0..v).map(|_| k.alias()).collect::<String>())
                    .sorted()
                    .collect::<Vec<_>>()
                    .concat();
                let mapped_eggs = cell
                    .eggs
                    .iter()
                    .map(|e| e.team_name.get(..1).unwrap())
                    .collect::<Vec<_>>()
                    .concat();
                let mapped_player = cell
                    .players
                    .iter()
                    .map(|p| {
                        format!(
                            "[{}{}]",
                            p,
                            direction_to_emoji(&players.get(p).unwrap().position().direction)
                        )
                    })
                    .collect::<String>();
                let widget = Paragraph::new(format!(
                    "{mapped_player}, {mapped_eggs}, {mapped_resources}"
                ))
                .block(Block::bordered().title(format!("y={y} x={x}")));
                frame.render_widget(widget, col);
            }
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut terminal = ratatui::init();

    let (event_tx, mut event_rx) = mpsc::channel(100);
    let (tx, mut rx) = mpsc::channel(100);
    let (conn_tx, mut conn_rx) = mpsc::channel(10);

    tokio::spawn(async move {
        loop {
            let poll = tokio::task::spawn_blocking(|| event::poll(Duration::from_millis(500)))
                .await
                .unwrap();

            if let Ok(true) = poll {
                let evt = tokio::task::spawn_blocking(|| event::read()).await.unwrap();
                if let Ok(Event::Key(key)) = evt {
                    if event_tx.send(key).await.is_err() {
                        break;
                    }
                }
            }
            tokio::time::sleep(Duration::from_millis(500)).await;
        }
    });

    tokio::spawn(async move {
        loop {
            match TcpStream::connect(format!("127.0.0.1:{}", GFX_PORT)).await {
                Ok(stream) => {
                    eprintln!("Connected to server");
                    let _ = conn_tx.send(true).await; // Notify connection
                    let reader = BufReader::new(stream);
                    let mut lines = reader.lines();

                    while let Ok(Some(line)) = lines.next_line().await {
                        match from_str::<Value>(&line) {
                            Ok(json_data) => {
                                let map: Result<Map, _> =
                                    serde_json::from_value(json_data["map"].clone());
                                let players: Result<HashMap<u16, Player>, _> =
                                    serde_json::from_value(json_data["players"].clone());
                                if let (Ok(map), Ok(players)) = (map, players) {
                                    if tx.send((map, players)).await.is_err() {
                                        break;
                                    }
                                } else {
                                    eprintln!("Failed to deserialize JSON");
                                }
                            }
                            Err(e) => eprintln!("Failed to deserialize JSON: {}", e),
                        }
                    }
                    eprintln!("Connection lost, retrying...");
                }
                Err(e) => {
                    eprintln!("Failed to connect: {}, retrying in 1 second...", e);
                    let _ = conn_tx.send(false).await;
                }
            }
            tokio::time::sleep(Duration::from_secs(1)).await;
        }
    });

    let mut map: Option<Map> = None;
    let mut players: Option<HashMap<u16, Player>> = None;

    loop {
        terminal.draw(|frame| {
            draw(frame, &mut map, &mut players);
        })?;

        tokio::select! {
            Some(key) = event_rx.recv() => {
                if key.code == KeyCode::Char('q') {
                    break;
                }
            }
            Some(new_data) = rx.recv() => {
                let all: (Map, HashMap<u16, Player>) = new_data;
                map = Some(all.0);
                players = Some(all.1);
            }
            Some(is_connected) = conn_rx.recv() => {
                if is_connected {
                    terminal.clear()?;
                }
            }
            //_ = tokio::time::sleep(Duration::from_millis(50)) => {}
        }
    }
    ratatui::restore();
    Ok(())
}
