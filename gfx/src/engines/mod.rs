use clap::ValueEnum;
use crossterm::event::KeyEvent;
use shared::map::Map;
use shared::player::Player;
use std::collections::BTreeMap;
use std::fmt::Debug;
use tokio::sync::mpsc::Receiver;

mod console;

use ratatui::style::Color as RatatuiColor;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum ZappyColor {
    Red,
    Green,
    Yellow,
    Blue,
    Magenta,
    Cyan,
    Gray,
    DarkGray,
    LightRed,
    LightGreen,
    LightYellow,
    LightBlue,
    LightMagenta,
    LightCyan,
}

impl ZappyColor {
    pub fn to_ratatui_value(&self) -> RatatuiColor {
        match self {
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
}

pub struct ServerData {
    pub map: Map,
    pub players: BTreeMap<u16, Player>,
    pub teams: BTreeMap<String, (ZappyColor, usize)>,
}

impl ServerData {
    const COLORS: [ZappyColor; 14] = [
        ZappyColor::Red,
        ZappyColor::Green,
        ZappyColor::Yellow,
        ZappyColor::Blue,
        ZappyColor::Magenta,
        ZappyColor::Cyan,
        ZappyColor::Gray,
        ZappyColor::DarkGray,
        ZappyColor::LightRed,
        ZappyColor::LightGreen,
        ZappyColor::LightYellow,
        ZappyColor::LightBlue,
        ZappyColor::LightMagenta,
        ZappyColor::LightCyan,
    ];

    pub fn new(map: Map, players: BTreeMap<u16, Player>, teams: BTreeMap<String, usize>) -> Self {
        let teams = teams
            .iter()
            .enumerate()
            .map(|(i, (name, &members_count))| {
                (
                    name.clone(),
                    (
                        ServerData::COLORS[i % ServerData::COLORS.len()],
                        members_count,
                    ),
                )
            })
            .collect::<BTreeMap<String, (ZappyColor, usize)>>();
        Self {
            map,
            players,
            teams,
        }
    }
}

#[derive(ValueEnum, Clone, Debug)]
pub enum Engine {
    Console,
    GUI,
}

impl Engine {
    pub async fn render(
        &self,
        event_rx: Receiver<KeyEvent>,
        rx: Receiver<ServerData>,
        conn_rx: Receiver<bool>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        match self {
            Engine::Console => console::render(event_rx, rx, conn_rx),
            Engine::GUI => console::render(event_rx, rx, conn_rx),
        }
        .await
    }
}
