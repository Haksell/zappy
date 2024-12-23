// TODO: if enough lines, one line for each team

use crate::Message;
use bevy::tasks::futures_lite::StreamExt as _;
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
    cell::CellPos,
    color::ZappyColor,
    player::Player,
    resource::{Resource, Stone, NOURRITURE_COLOR},
    GFXData,
};
use std::collections::{BTreeMap, VecDeque};
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

fn team_color(data: &GFXData, team: &String) -> RatatuiColor {
    zappy_to_ratatui_color(data.teams.get(team).unwrap().0)
}

fn map_resource_to_vec_span<'a>(
    nourriture: &'a VecDeque<CellPos>,
    stones: &'a [VecDeque<CellPos>; Stone::SIZE],
) -> Vec<Span<'a>> {
    let mut spans = Vec::new();

    // Add nourriture spans
    let nourriture_color = NOURRITURE_COLOR;
    let nourriture_style = Style::default()
        .fg(zappy_to_ratatui_color(nourriture_color))
        .bold();
    spans.extend(vec![Span::styled("N", nourriture_style); nourriture.len()]);

    // Add resource spans
    for (i, cnt) in stones.iter().enumerate() {
        if cnt.is_empty() {
            continue;
        }

        let resource = Resource::try_from(i).unwrap();
        let style = Style::default()
            .fg(zappy_to_ratatui_color(resource.color()))
            .bold();
        let resource_str = resource.alias().to_string().repeat(cnt.len());
        spans.push(Span::styled(resource_str, style));
    }

    spans
}

fn map_stones_to_vec_span(resources: &[usize; Stone::SIZE]) -> Vec<Span> {
    resources
        .iter()
        .enumerate()
        .map(|(i, &cnt)| {
            if cnt == 0 {
                return Span::raw("");
            }
            let resource = Resource::try_from(i).unwrap();
            Span::styled(
                resource.alias().to_string().repeat(cnt),
                Style::default()
                    .fg(zappy_to_ratatui_color(resource.color()))
                    .bold(),
            )
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
            player.position().dir.as_char(),
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

fn draw_field(data: &GFXData, frame: &mut Frame, area: Rect) {
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
            let mapped_map_resources = map_resource_to_vec_span(&cell.nourriture, &cell.stones);
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
                .keys()
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

fn draw_players_bar(data: &GFXData, frame: &mut Frame, area: Rect) {
    let block = Block::default()
        .title("Players Stats")
        .borders(Borders::ALL);

    let inner_area = block.inner(area);
    frame.render_widget(block, area);

    let rows = Layout::vertical(vec![Constraint::Length(3); data.teams.len()]).split(inner_area);

    let mut teams_data = data
        .teams
        .iter()
        .map(|(team_name, &(team_color, _))| {
            (
                team_name.clone(),
                vec![vec![Span::styled(
                    team_name,
                    Style::default().fg(zappy_to_ratatui_color(team_color)),
                )]],
            )
        })
        .collect::<BTreeMap<String, Vec<Vec<Span>>>>();

    for player in data.players.values() {
        if let Some(details) = teams_data.get_mut(player.team()) {
            let mut player_details: Vec<Span> = Vec::new();
            let style = Style::default().fg(team_color(data, player.team()));
            player_details.push(Span::styled(format!("🧬 {}", player.id()), style));
            player_details.push(Span::raw(" | "));
            player_details.push(Span::styled(
                format!("💜 {}", player.remaining_life()),
                style,
            ));
            player_details.push(Span::raw(" | "));
            player_details.push(Span::raw("⭐".repeat(*player.level() as usize)));
            player_details.push(Span::raw(" | 🎒 "));
            player_details.extend(map_stones_to_vec_span(player.inventory()));
            player_details.push(Span::raw(" |"));

            details.push(player_details);
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
    let mut prev_state: Option<GFXData> = None;
    let mut event_stream = crossterm::event::EventStream::new();

    loop {
        tokio::select! {
            event = event_stream.next() => {
                if let Some(Ok(event)) = event {
                    match event {
                        crossterm::event::Event::Key(key) => {
                            if key.code == KeyCode::Esc || key.code == KeyCode::Char('q') || key.code == KeyCode::Char('Q') {
                                break;
                            }
                        },
                        crossterm::event::Event::Resize(_, _) => {
                            if let Some(state) = &prev_state {
                                terminal.draw(|frame| {
                                    let layout =
                                        Layout::vertical([Constraint::Percentage(80), Constraint::Percentage(20)])
                                            .split(frame.area());

                                    draw_field(&state, frame, layout[0]);
                                    draw_players_bar(&state, frame, layout[1]);
                                })?;
                            }
                        },
                        _ => {},
                    }
                }
            }
            message = data_rx.recv() => {
                let message = match message {
                    Some(message) => message,
                    None => {
                        eprintln!("None in recv ????");
                        continue;
                    }
                };
                match message {
                    Message::Disconnect(error) => {
                        if prev_state.is_some() {
                            terminal.clear()?;
                            prev_state = None;
                        }
                        terminal.draw(|frame| {
                            let layout = Layout::vertical([Constraint::Percentage(80), Constraint::Percentage(20)])
                                .split(frame.area());

                            let error_widget = Paragraph::new(format!("Failed to connect: {}, retrying in 1 second...", error))
                                .block(Block::default().borders(Borders::ALL).title("Error"))
                                .style(Style::default().fg(Color::Red));
                            frame.render_widget(error_widget, layout[1]);
                        })?;
                    }
                    Message::State(new_state) => {
                        if prev_state.is_none() {
                            terminal.clear()?;
                        }
                        terminal.draw(|frame| {
                            let layout =
                                Layout::vertical([Constraint::Percentage(80), Constraint::Percentage(20)])
                                    .split(frame.area());

                            draw_field(&new_state, frame, layout[0]);
                            draw_players_bar(&new_state, frame, layout[1]);
                            prev_state = Some(new_state);
                        })?;
                    }
                }
            }
        }
    }

    ratatui::restore();
    Ok(())
}
