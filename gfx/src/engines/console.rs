// TODO: if enough lines, one line for each team

use crate::{engines::ServerData, Message};
use itertools::Itertools as _;
use ratatui::{
    crossterm::event::KeyCode,
    layout::{Constraint, Layout},
    prelude::{Alignment, Color, Line, Rect, Span, Style, Stylize},
    style::Color as RatatuiColor,
    widgets::Block,
    widgets::{BorderType, Borders, Paragraph, Wrap},
    Frame,
};
use shared::{
    color::ZappyColor,
    player::Player,
    position::Direction,
    resource::{Resource, Stone, NOURRITURE_COLOR},
};
use std::collections::BTreeMap;
use tokio::sync::mpsc::UnboundedReceiver;

fn zappy_to_ratatui_color(color: ZappyColor) -> RatatuiColor {
    match color {
        ZappyColor::Red => RatatuiColor::Red,
        ZappyColor::Green => RatatuiColor::Green,
        ZappyColor::Yellow => RatatuiColor::Yellow,
        ZappyColor::Blue => RatatuiColor::Blue,
        ZappyColor::Magenta => RatatuiColor::Magenta,
        ZappyColor::Cyan => RatatuiColor::Cyan,
        ZappyColor::Gray => RatatuiColor::Gray,
        ZappyColor::DarkGray => RatatuiColor::DarkGray,
        ZappyColor::LightRed => RatatuiColor::LightRed,
        ZappyColor::LightGreen => RatatuiColor::LightGreen,
        ZappyColor::LightYellow => RatatuiColor::LightYellow,
        ZappyColor::LightBlue => RatatuiColor::LightBlue,
        ZappyColor::LightMagenta => RatatuiColor::LightMagenta,
        ZappyColor::LightCyan => RatatuiColor::LightCyan,
    }
}

// TODO: char
fn direction_to_emoji(direction: &Direction) -> &'static str {
    match direction {
        Direction::North => "^",
        Direction::East => ">",
        Direction::South => "v",
        Direction::West => "<",
    }
}

fn map_resource_to_vec_span(nourriture: usize, stones: &[usize; Stone::SIZE]) -> Vec<Span> {
    let mut spans = Vec::new();

    // Add nourriture spans
    let nourriture_color = NOURRITURE_COLOR;
    let nourriture_style = Style::default()
        .fg(zappy_to_ratatui_color(nourriture_color))
        .bold();
    spans.extend(vec![Span::styled("N", nourriture_style); nourriture]);

    // Add resource spans
    for (i, &count) in stones.iter().enumerate() {
        if count == 0 {
            continue;
        }

        let color = ZappyColor::idx(i);
        let style = Style::default().fg(zappy_to_ratatui_color(color)).bold();
        let char = Resource::try_from(i).unwrap().alias();
        let resource_str = char.to_string().repeat(count);
        spans.push(Span::styled(resource_str, style));
    }

    spans
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
                        .fg(zappy_to_ratatui_color(ZappyColor::idx(i)))
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
            "[{}{}{}]",
            if *player.is_performing_incantation() {
                "🗿"
            } else {
                ""
            },
            player.id(),
            direction_to_emoji(&player.position().dir),
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
            let mapped_map_resources = map_resource_to_vec_span(cell.nourriture, &cell.stones);
            let mapped_eggs = cell
                .eggs
                .iter()
                .map(|(team_name, &eggs)| {
                    let color = data.teams.get(team_name).unwrap().0;
                    eggs_to_span(eggs, zappy_to_ratatui_color(color))
                })
                .collect::<Vec<_>>();
            let mapped_player = cell
                .players
                .iter()
                .sorted()
                .map(|p| {
                    let player = data.players.get(p).unwrap();
                    map_player_to_span(
                        zappy_to_ratatui_color(data.teams.get(player.team()).unwrap().0),
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
            let team_color = zappy_to_ratatui_color(data.teams.get(team_name).unwrap().0);
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
            let style = Style::default().fg(zappy_to_ratatui_color(
                data.teams.get(player.team()).unwrap().0,
            ));
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
    mut data_rx: UnboundedReceiver<Message>,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut terminal = ratatui::init();

    loop {
        let message = match data_rx.recv().await {
            Some(message) => message,
            None => {
                eprintln!("None in recv ????");
                continue;
            }
        };
        match message {
            Message::Disconnect => todo!(),
            Message::KeyEvent(key) => {
                if key.code == KeyCode::Esc
                    || key.code == KeyCode::Char('q')
                    || key.code == KeyCode::Char('Q')
                {
                    break;
                }
            }
            Message::Data(new_data) => {
                terminal.clear()?; // TODO Test on connect ?
                terminal.draw(|frame| {
                    let layout =
                        Layout::vertical([Constraint::Percentage(80), Constraint::Percentage(20)])
                            .split(frame.area());

                    draw_field(&new_data, frame, layout[0]);
                    draw_players_bar(&new_data, frame, layout[1]);
                })?;
            }
        }
    }
    ratatui::restore();
    Ok(())
}
