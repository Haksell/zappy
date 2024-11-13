// use clap::ValueEnum;
// use crossterm::event::KeyEvent;
// use shared::player::Player;
// use shared::Map;
// use std::collections::HashMap;
// use std::fmt::Debug;
// use tokio::sync::mpsc::Receiver;

use std::collections::HashMap;

use clap::ValueEnum;
use crossterm::event::KeyEvent;
use shared::{player::Player, Map};
use tokio::sync::mpsc::Receiver;

mod console;
mod torus;

#[derive(ValueEnum, Clone, Debug)]
pub enum Engine {
    Console,
    Torus,
}

impl Engine {
    pub async fn render(
        &self,
        event_rx: Receiver<KeyEvent>,
        rx: Receiver<(Map, HashMap<u16, Player>)>,
        conn_rx: Receiver<bool>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        match self {
            Engine::Console => console::render(event_rx, rx, conn_rx).await,
            Engine::Torus => torus::render(event_rx, rx, conn_rx).await,
        }
    }
}
