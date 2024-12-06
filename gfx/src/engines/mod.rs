mod console;
mod torus;

use super::Message;
use clap::ValueEnum;
use shared::{color::ZappyColor, map::Map, player::Player};
use std::{collections::BTreeMap, fmt::Debug};
use tokio::sync::mpsc::UnboundedReceiver;

#[derive(ValueEnum, Clone, Debug, PartialEq, Eq)]
pub enum Engine {
    Console,
    Torus,
}

impl Engine {
    pub async fn render(
        &self,
        data_rx: UnboundedReceiver<Message>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        match self {
            Engine::Console => console::render(data_rx).await,
            Engine::Torus => torus::render(data_rx).await,
        }
    }
}
