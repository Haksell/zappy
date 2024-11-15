use crossterm::event::KeyEvent;
use ratatui::widgets::{BorderType, Borders, Paragraph};
use ratatui::{crossterm::event::KeyCode, widgets::Block};
use ratatui::{
    layout::{Constraint, Layout},
    Frame,
};

use crate::engines::ServerData;
use itertools::Itertools;
use ratatui::prelude::{Color, Line, Rect, Span, Style};
use shared::player::{Direction, Player};
use shared::resource::Resource;
use std::collections::HashMap;
use tokio::sync::mpsc::Receiver;

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

fn map_resource_to_vec_span(resources: &[usize; 7]) -> Vec<Span> {
    resources
        .iter()
        .enumerate()
        .map(|(i, &cnt)| {
            let c = Resource::try_from(i as u8).unwrap().alias();
            let resource_str = (0..cnt).map(|_| c).collect::<String>();
            if !resource_str.is_empty() {
                Span::styled(
                    resource_str,
                    Style::default()
                        .fg(ServerData::COLORS[i % ServerData::COLORS.len()].to_ratatui_value()),
                )
            } else {
                Span::raw("")
            }
        })
        .collect::<Vec<Span>>()
}

fn map_player_to_span(color: Color, player: &Player) -> Span {
    Span::styled(
        format!(
            "[{}{}]",
            player.id(),
            direction_to_emoji(&player.position().direction),
        ),
        Style::default().fg(color),
    )
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

fn draw_field(data: &ServerData, frame: &mut Frame, area: Rect) {
    let rows = Layout::vertical(vec![
        Constraint::Ratio(1, data.map.height as u32);
        data.map.height
    ])
    .split(area);

    let mut cols = rows.iter().flat_map(|row| {
        Layout::horizontal(vec![
            Constraint::Ratio(1, data.map.width as u32);
            data.map.width
        ])
        .split(*row)
        .to_vec()
    });

    for y in 0..data.map.height {
        for x in 0..data.map.width {
            let col = cols.next().unwrap();
            let cell = &data.map.field[y][x];
            let mapped_map_resources = map_resource_to_vec_span(&cell.resources);
            let mapped_eggs = cell
                .eggs
                .iter()
                .map(|e| e.team_name.get(..1).unwrap())
                .collect::<Vec<_>>()
                .concat();
            let mapped_player = cell
                .players
                .iter()
                .sorted()
                .map(|p| {
                    let player = data.players.get(p).unwrap();
                    map_player_to_span(
                        data.teams.get(player.team()).unwrap().0.to_ratatui_value(),
                        player,
                    )
                })
                .collect::<Vec<_>>();

            let mut spans = vec![];
            spans.extend(mapped_player);
            spans.push(Span::raw(format!(", {mapped_eggs}")));

            if !mapped_map_resources.is_empty() {
                spans.push(Span::raw(", "));
                spans.extend(mapped_map_resources);
            }

            let widget = Paragraph::new(Line::from(spans))
                .block(Block::bordered().title(format!("y={y} x={x}")));
            frame.render_widget(widget, col);
        }
    }
}

fn draw_players_bar(data: ServerData, frame: &mut Frame, area: Rect) {
    let block = Block::default()
        .title("Players Stats")
        .borders(Borders::ALL);

    let inner_area = block.inner(area);
    frame.render_widget(block, area);

    let rows = Layout::vertical(vec![
        Constraint::Ratio(1, data.teams.len() as u32);
        data.teams.len()
    ])
    .split(inner_area);

    //let mut parsed_teams = HashMap::with_capacity(data.teams.len());

    for p in data.players {}

    let teams_data = vec![
        (Color::Red, vec!["team 1", "player 1"]),
        (
            Color::Blue,
            vec!["team 2", "player 2", "player 3", "player 4"],
        ),
        (Color::Green, vec!["team 3", "player 5", "player 6"]),
    ];

    for (i, row) in rows.iter().enumerate() {
        if i < teams_data.len() {
            let (team_color, team_members) = &teams_data[i];
            let mut constraints = vec![Constraint::Length(15)];
            constraints.extend(vec![
                Constraint::Ratio(1, (team_members.len() - 1) as u32);
                team_members.len() - 1
            ]);

            let cols = Layout::horizontal(constraints).split(*row);

            for (col_idx, col) in cols.iter().enumerate() {
                let text = team_members[col_idx];
                let spans: Vec<Span> = text
                    .chars()
                    .enumerate()
                    .map(|(i, c)| {
                        let color = match i % 3 {
                            0 => *team_color,
                            1 => Color::Yellow,
                            _ => Color::White,
                        };
                        Span::styled(c.to_string(), Style::default().fg(color))
                    })
                    .collect();

                let cell = Paragraph::new(Line::from(spans)).block(
                    Block::default()
                        .borders(Borders::ALL)
                        .border_type(BorderType::Plain),
                );

                frame.render_widget(cell, *col);
            }
        }
    }
}

pub async fn render(
    mut event_rx: Receiver<KeyEvent>,
    mut rx: Receiver<ServerData>,
    mut conn_rx: Receiver<bool>,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut terminal = ratatui::init();
    let mut data: Option<ServerData> = None;

    loop {
        terminal.draw(|frame| {
            let layout = Layout::vertical([Constraint::Percentage(80), Constraint::Percentage(20)])
                .split(frame.area());

            if let Some(data) = &data {
                draw_field(data, frame, layout[0]);
            }
            /*
            draw_players_bar(
                frame,
                &mut map,
                &mut players,
                &mut teams,
                &team_colors,
                layout[1],
            );
             */
        })?;

        tokio::select! {
            Some(key) = event_rx.recv() => {
                if key.code == KeyCode::Char('q') {
                    break;
                }
            }
            Some(new_data) = rx.recv() => {
                data = Some(new_data);
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
