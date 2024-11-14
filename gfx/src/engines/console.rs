use crossterm::event::KeyEvent;
use ratatui::layout::Margin;
use ratatui::widgets::{BorderType, Borders, Paragraph, Wrap};
use ratatui::{crossterm::event::KeyCode, widgets::Block};
use ratatui::{
    layout::{Constraint, Layout},
    Frame,
};

use itertools::Itertools;
use ratatui::prelude::{Color, Line, Rect, Span, Style, Stylize};
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

struct ResourceSpans([usize; 7]);

struct Data {
    pub map: Map,
    pub players: HashMap<u16, Player>,
    pub teams: Vec<(String, usize)>,
    pub team_colors: HashMap<String, Color>,
}

impl Data {
    const COLORS: [Color; 14] = [
        Color::Red,
        Color::Green,
        Color::Yellow,
        Color::Blue,
        Color::Magenta,
        Color::Cyan,
        Color::Gray,
        Color::DarkGray,
        Color::LightRed,
        Color::LightGreen,
        Color::LightYellow,
        Color::LightBlue,
        Color::LightMagenta,
        Color::LightCyan,
    ];

    pub fn new(map: Map, players: HashMap<u16, Player>, teams: Vec<(String, usize)>) -> Self {
        let team_colors = teams
            .iter()
            .enumerate()
            .map(|(i, (name, _))| (name.clone(), Data::COLORS[i]))
            .collect::<HashMap<String, Color>>();
        Self {
            map,
            players,
            teams,
            team_colors,
        }
    }
}

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
                    Style::default().fg(Data::COLORS[i % Data::COLORS.len()]),
                )
            } else {
                Span::raw("")
            }
        })
        .collect::<Vec<Span>>()
}

fn map_player_to_span(color: Color, player: &Player) -> Span {
    Span::styled(format!(
        "[{}{}]",
        player.id(),
        direction_to_emoji(&player.position().direction),
    ), Style::default().fg(color))
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

fn draw_field(data: &Data, frame: &mut Frame, area: Rect) {
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
                .map(|p| {
                    let player = data.players.get(p).unwrap();
                        map_player_to_span(*data.team_colors.get(player.team()).unwrap(), player)
                    
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

fn draw_players_bar(
    data: Data,
    frame: &mut Frame,
    area: Rect,
) {
        let block = Block::default()
            .title("Players Stats")
            .borders(Borders::ALL);

        let inner_area = block.inner(area);
        frame.render_widget(block, area);

        let rows = Layout::vertical(vec![Constraint::Ratio(1, data.teams.len() as u32); data.teams.len()])
            .split(inner_area);

        for (i, row) in rows.iter().enumerate() {
                let mut constraints = vec![Constraint::Length(15)];
                constraints.extend(vec![
                    Constraint::Ratio(1, (data.teams.len() - 1) as u32);
                    data.teams.len() - 1
                ]);

                let cols = Layout::horizontal(constraints).split(*row);

                for (col_idx, col) in cols.iter().enumerate() {
                    let text = col_idx.to_string();
                    let spans: Vec<Span> = text
                        .chars()
                        .enumerate()
                        .map(|(i, c)| {
                            let color = match i % 3 {
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

pub async fn render(
    mut event_rx: Receiver<KeyEvent>,
    mut rx: Receiver<(Map, HashMap<u16, Player>, Vec<(String, usize)>)>,
    mut conn_rx: Receiver<bool>,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut terminal = ratatui::init();
    let mut data: Option<Data> = None;

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
                let all: (Map, HashMap<u16, Player>, Vec<(String, usize)>) = new_data;
                data = Some(Data::new(all.0, all.1, all.2));
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
