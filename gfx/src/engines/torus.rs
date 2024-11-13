use crossterm::event::KeyEvent;
use shared::player::Player;
use shared::Map;
use std::collections::HashMap;
use tokio::sync::mpsc::Receiver;

pub async fn render(
    mut event_rx: Receiver<KeyEvent>,
    mut rx: Receiver<(Map, HashMap<u16, Player>)>,
    mut conn_rx: Receiver<bool>,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("torus");
    Ok(())
}
