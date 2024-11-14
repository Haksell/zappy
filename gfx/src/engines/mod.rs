use clap::ValueEnum;
use crossterm::event::KeyEvent;
use shared::map::Map;
use shared::player::Player;
use std::collections::HashMap;
use std::fmt::Debug;
use tokio::sync::mpsc::Receiver;

mod console;

#[derive(ValueEnum, Clone, Debug)]
pub enum Engine {
    Console,
    GUI,
}

impl Engine {
    pub async fn render(
        &self,
        event_rx: Receiver<KeyEvent>,
        rx: Receiver<(Map, HashMap<u16, Player>, Vec<(String, usize)>)>,
        conn_rx: Receiver<bool>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        match self {
            Engine::Console => console::render(event_rx, rx, conn_rx),
            Engine::GUI => console::render(event_rx, rx, conn_rx),
        }
        .await
    }
}
