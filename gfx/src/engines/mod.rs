mod console;
mod torus;

use clap::ValueEnum;
use crossterm::event::KeyEvent;
use ratatui::style::Color as RatatuiColor;
use shared::{map::Map, player::Player};
use std::{collections::BTreeMap, fmt::Debug};
use tokio::sync::mpsc::Receiver;

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

    pub fn _to_rgb(&self) -> (u8, u8, u8) {
        match self {
            ZappyColor::Red => (255, 0, 0),
            ZappyColor::Green => (0, 255, 0),
            ZappyColor::Yellow => (255, 255, 0),
            ZappyColor::Blue => (0, 0, 255),
            ZappyColor::Magenta => (255, 0, 255),
            ZappyColor::Cyan => (0, 255, 255),
            ZappyColor::Gray => (128, 128, 128),
            ZappyColor::DarkGray => (64, 64, 64),
            ZappyColor::LightRed => (255, 128, 128),
            ZappyColor::LightGreen => (128, 255, 128),
            ZappyColor::LightYellow => (255, 255, 128),
            ZappyColor::LightBlue => (128, 128, 255),
            ZappyColor::LightMagenta => (255, 128, 255),
            ZappyColor::LightCyan => (128, 255, 255),
        }
    }
}

#[derive(Debug, Default)]
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
                (name.clone(), (ServerData::color(i), members_count))
            })
            .collect::<BTreeMap<String, (ZappyColor, usize)>>();
        Self {
            map,
            players,
            teams,
        }
    }

    pub fn color(color_idx: usize) -> ZappyColor {
        Self::COLORS[color_idx % Self::COLORS.len()]
    }
}

#[derive(ValueEnum, Clone, Debug)]
pub enum Engine {
    Console,
    Torus,
}

impl Engine {
    pub async fn render(
        &self,
        event_rx: Receiver<KeyEvent>,
        data_rx: Receiver<ServerData>,
        conn_rx: Receiver<bool>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        match self {
            Engine::Console => console::render(event_rx, data_rx, conn_rx).await,
            Engine::Torus => torus::render(data_rx).await,
        }
    }
}
