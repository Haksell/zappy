mod console;
mod torus;

use clap::ValueEnum;
use crossterm::event::KeyEvent;
use shared::{color::ZappyColor, map::Map, player::Player};
use std::{collections::BTreeMap, fmt::Debug};
use tokio::sync::mpsc::Receiver;

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
