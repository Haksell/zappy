// TODO: if enough lines, one line for each team

use crate::engines::ServerData;
use crossterm::event::KeyEvent;
use itertools::Itertools as _;
use ratatui::{
    crossterm::event::KeyCode,
    layout::{Constraint, Layout},
    prelude::{Alignment, Color, Line, Rect, Span, Style, Stylize},
    widgets::Block,
    widgets::{BorderType, Borders, Paragraph, Wrap},
    Frame,
};
use shared::{
    player::Player,
    position::Direction,
    resource::{Resource, Stone},
};
use std::collections::BTreeMap;
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

fn map_resource_to_vec_span(resources: &[usize; Resource::SIZE]) -> Vec<Span> {
    resources
        .iter()
        .enumerate()
        .map(|(i, &cnt)| {
            let c = Resource::try_from(i).unwrap().alias();
            let resource_str = (0..cnt).map(|_| c).collect::<String>();
            if !resource_str.is_empty() {
                Span::styled(
                    resource_str,
                    Style::default()
                        .fg(ServerData::COLORS[i % ServerData::COLORS.len()].to_ratatui_value())
                        .bold(),
                )
            } else {
                Span::raw("")
            }
        })
        .collect::<Vec<Span>>()
}

fn map_stones_to_vec_span(resources: &[usize; Stone::SIZE]) -> Vec<Span> {
    resources
        .iter()
        .enumerate()
        .map(|(i, &cnt)| {
            let c = Resource::try_from(i).unwrap().alias();
            let resource_str = (0..cnt).map(|_| c).collect::<String>();
            if !resource_str.is_empty() {
                Span::styled(
                    resource_str,
                    Style::default()
                        .fg(ServerData::COLORS[i % ServerData::COLORS.len()].to_ratatui_value())
                        .bold(),
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

fn eggs_to_span(eggs: (usize, usize), color: Color) -> Span<'static> {
    let s = match eggs {
        (0, 0) => "".to_string(),
        (unhatched, hatched) => format!(
            "({}{})",
            "🥚".to_string().repeat(unhatched),
            "🐣".to_string().repeat(hatched),
        ),
    };
    Span::styled(s, Style::default().fg(color))
}

fn draw_field(data: &ServerData, frame: &mut Frame, area: Rect) {
    let rows = Layout::vertical(vec![
        Constraint::Ratio(1, *data.map.height() as u32);
        *data.map.height()
    ])
    .split(area);

    let mut cols = rows.iter().flat_map(|row| {
        Layout::horizontal(vec![
            Constraint::Ratio(1, *data.map.width() as u32);
            *data.map.width()
        ])
        .split(*row)
        .to_vec()
    });

    for y in 0..*data.map.height() {
        for x in 0..*data.map.width() {
            let col = cols.next().unwrap();
            let cell = &data.map.field[y][x];
            let mapped_map_resources = map_resource_to_vec_span(&cell.resources);
            let mapped_eggs = cell
                .eggs
                .iter()
                .map(|(team_name, &eggs)| {
                    let color = data.teams.get(team_name).unwrap().0;
                    eggs_to_span(eggs, color.to_ratatui_value())
                })
                .collect::<Vec<_>>();
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
            for vec in [mapped_player, mapped_eggs, mapped_map_resources] {
                if !vec.is_empty() {
                    if !spans.is_empty() {
                        spans.push(Span::raw(" "));
                    }
                    spans.extend(vec);
                }
            }

            let widget = Paragraph::new(Line::from(spans))
                .block(Block::bordered().title(format!("y={y} x={x}")))
                .wrap(Wrap { trim: true });
            frame.render_widget(widget, col);
        }
    }
}

fn draw_players_bar(data: &ServerData, frame: &mut Frame, area: Rect) {
    let block = Block::default()
        .title("Players Stats")
        .borders(Borders::ALL);

    let inner_area = block.inner(area);
    frame.render_widget(block, area);

    let rows = Layout::vertical(vec![Constraint::Length(3); data.teams.len()]).split(inner_area);

    let mut teams_data = data
        .teams
        .keys()
        .map(|team_name| {
            //TODO: make function that return color to avoid unwrap and .0 every time
            let team_color = data.teams.get(team_name).unwrap().0.to_ratatui_value();
            (
                team_name.clone(),
                vec![vec![Span::styled(
                    team_name,
                    Style::default().fg(team_color),
                )]],
            )
        })
        .collect::<BTreeMap<String, Vec<Vec<Span>>>>();

    for player in data.players.values() {
        if let Some(details) = teams_data.get_mut(player.team()) {
            let mut current_player_details: Vec<Span> = Vec::new();
            let style =
                Style::default().fg(data.teams.get(player.team()).unwrap().0.to_ratatui_value());
            current_player_details.push(Span::styled(format!("🧬 {}", player.id()), style));
            current_player_details.push(Span::raw(" | "));
            current_player_details.push(Span::styled(
                format!("💜 {}", player.remaining_life()),
                style,
            ));
            current_player_details.push(Span::raw(" | "));
            current_player_details.push(Span::raw("⭐".repeat(*player.level() as usize)));
            current_player_details.push(Span::raw(" | 🎒 "));
            current_player_details.extend(map_stones_to_vec_span(player.inventory()));
            current_player_details.push(Span::raw(" |"));

            details.push(current_player_details);
        }
    }

    for (i, (_, member_details)) in teams_data.iter().enumerate() {
        if i < rows.len() {
            let mut constraints = vec![Constraint::Length(10)];
            constraints.extend(vec![
                Constraint::Ratio(1, (member_details.len() - 1) as u32);
                member_details.len().max(1) - 1
            ]);

            let cols = Layout::horizontal(constraints).split(rows[i]);

            for (col_idx, col) in cols.iter().enumerate() {
                if col_idx < member_details.len() {
                    let cell = Paragraph::new(Line::from(member_details[col_idx].clone()))
                        .block(
                            Block::default()
                                .borders(Borders::ALL)
                                .border_type(BorderType::Plain),
                        )
                        .alignment(Alignment::Center);

                    frame.render_widget(cell, *col);
                }
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
                draw_players_bar(data, frame, layout[1]);
            }
        })?;

        tokio::select! {
            Some(key) = event_rx.recv() => {
                if key.code == KeyCode::Esc || key.code == KeyCode::Char('q') || key.code == KeyCode::Char('Q') {
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
