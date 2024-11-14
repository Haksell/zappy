use crossterm::event::KeyEvent;
use ratatui::layout::Margin;
use ratatui::widgets::Paragraph;
use ratatui::{crossterm::event::KeyCode, widgets::Block};
use ratatui::{
    layout::{Constraint, Layout},
    Frame,
};
use shared::map::Map;
use shared::player::{Direction, Player};
use shared::resource::Resource;
use std::collections::HashMap;
use tokio::sync::mpsc::Receiver;
// pub const NORTH_EMOJI: &'static str = "↥";
// pub const EAST_EMOJI: &'static str = "↦";
// pub const SOUTH_EMOJI: &'static str = "↧";
// pub const WEST_EMOJI: &'static str = "↤";

pub const NORTH_EMOJI: &'static str = "^";
pub const EAST_EMOJI: &'static str = ">";
pub const SOUTH_EMOJI: &'static str = "v";
pub const WEST_EMOJI: &'static str = "<";

fn direction_to_emoji(direction: &Direction) -> &'static str {
    match direction {
        Direction::North => NORTH_EMOJI,
        Direction::East => EAST_EMOJI,
        Direction::South => SOUTH_EMOJI,
        Direction::West => WEST_EMOJI,
    }
}

fn map_resource_chars(i: usize, cnt: &usize) -> impl Iterator<Item = char> {
    let c = Resource::try_from(i as u8).unwrap().alias();
    std::iter::repeat(c).take(*cnt)
}
fn map_player_inventory(players: &mut HashMap<u16, Player>, id: &u16) -> String {
    players
        .get(id)
        .unwrap()
        .inventory()
        .iter()
        .enumerate()
        .flat_map(|(i, cnt)| map_resource_chars(i, cnt))
        .collect()
}

fn draw(frame: &mut Frame, map: &mut Option<Map>, players: &mut Option<HashMap<u16, Player>>) {
    if let (Some(data), Some(players)) = (map, players) {
        let area = frame.area().inner(Margin {
            vertical: 1,
            horizontal: 1,
        });

        let rows = Layout::vertical(vec![Constraint::Ratio(1, data.height as u32); data.height])
            .split(area);

        let mut cols = rows.iter().flat_map(|row| {
            Layout::horizontal(vec![Constraint::Ratio(1, data.width as u32); data.width])
                .split(*row)
                .to_vec()
        });

        for y in 0..data.height {
            for x in 0..data.width {
                let col = cols.next().unwrap();
                let cell = &data.field[y][x];
                let mapped_map_resources = cell
                    .resources
                    .iter()
                    .enumerate()
                    .map(|(i, &cnt)| {
                        let c = Resource::try_from(i as u8).unwrap().alias();
                        (0..cnt).map(|_| c).collect::<String>()
                    })
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
                            "[{}{}({})]",
                            p,
                            direction_to_emoji(&players.get(p).unwrap().position().direction),
                            map_player_inventory(players, p)
                        )
                    })
                    .collect::<String>();

                let widget = Paragraph::new(format!(
                    "{mapped_player}, {mapped_eggs}, {mapped_map_resources}"
                ))
                .block(Block::bordered().title(format!("y={y} x={x}")));
                frame.render_widget(widget, col);
            }
        }
    }
}

pub async fn render(
    mut event_rx: Receiver<KeyEvent>,
    mut rx: Receiver<(Map, HashMap<u16, Player>)>,
    mut conn_rx: Receiver<bool>,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut terminal = ratatui::init();
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
