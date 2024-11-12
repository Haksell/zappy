use crossterm::event::{self, Event};
use serde_json::{from_str, Value};
use shared::player::Player;
use shared::Map;
use shared::GFX_PORT;
use std::collections::HashMap;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::net::TcpStream;
use tokio::sync::mpsc;
use tokio::time::Duration;

mod engines;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (event_tx, event_rx) = mpsc::channel(100);
    let (tx, rx) = mpsc::channel(100);
    let (conn_tx, conn_rx) = mpsc::channel(10);

    tokio::spawn(async move {
        loop {
            let poll = tokio::task::spawn_blocking(|| event::poll(Duration::from_millis(500)))
                .await
                .unwrap();

            if let Ok(true) = poll {
                let evt = tokio::task::spawn_blocking(|| event::read()).await.unwrap();
                if let Ok(Event::Key(key)) = evt {
                    if event_tx.send(key).await.is_err() {
                        break;
                    }
                }
            }
            tokio::time::sleep(Duration::from_millis(500)).await;
        }
    });

    tokio::spawn(async move {
        loop {
            // TODO Adress and port in program parameter, default localhost
            match TcpStream::connect(format!("127.0.0.1:{}", GFX_PORT)).await {
                Ok(stream) => {
                    eprintln!("Connected to server");
                    let _ = conn_tx.send(true).await; // Notify connection
                    let reader = BufReader::new(stream);
                    let mut lines = reader.lines();

                    while let Ok(Some(line)) = lines.next_line().await {
                        match from_str::<Value>(&line) {
                            Ok(json_data) => {
                                let map: Result<Map, _> =
                                    serde_json::from_value(json_data["map"].clone());
                                let players: Result<HashMap<u16, Player>, _> =
                                    serde_json::from_value(json_data["players"].clone());
                                if let (Ok(map), Ok(players)) = (map, players) {
                                    if tx.send((map, players)).await.is_err() {
                                        break;
                                    }
                                } else {
                                    eprintln!("Failed to deserialize JSON");
                                }
                            }
                            Err(e) => eprintln!("Failed to deserialize JSON: {}", e),
                        }
                    }
                    eprintln!("Connection lost, retrying...");
                }
                Err(e) => {
                    eprintln!("Failed to connect: {}, retrying in 1 second...", e);
                    let _ = conn_tx.send(false).await;
                }
            }
            tokio::time::sleep(Duration::from_secs(1)).await;
        }
    });

    engines::render(event_rx, rx, conn_rx).await
}
