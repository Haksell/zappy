use crate::engines::ServerData;
use clap::Parser;
use crossterm::event::{self, Event};
use engines::Engine;
use serde_json::{from_str, Value};
use shared::map::Map;
use shared::player::Player;
use shared::GFX_PORT;
use std::collections::BTreeMap;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::net::TcpStream;
use tokio::sync::mpsc;
use tokio::time::Duration;

mod engines;

#[derive(Parser, Debug)]
#[command(name = "gfx", about, long_about = None, about = "Graphical client for zappy.")]
struct Args {
    #[arg(short, long, default_value_t = String::from("127.0.0.1"), help = "Address of the server.")]
    address: String,

    #[arg(short, long, default_value_t = GFX_PORT, help = "Port of the server.")]
    port: u16,

    #[arg(short, long, value_enum, default_value_t = Engine::Torus, help = "Engine used for rendering.")]
    engine: Engine,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    let (event_tx, event_rx) = mpsc::channel(100); // TODO see console.rs 275
    let (data_tx, data_rx) = mpsc::unbounded_channel();

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
            match TcpStream::connect(format!("{}:{}", args.address, args.port)).await {
                Ok(stream) => {
                    eprintln!("Connected to server");
                    let reader = BufReader::new(stream);
                    let mut lines = reader.lines();

                    while let Ok(Some(line)) = lines.next_line().await {
                        match from_str::<Value>(&line) {
                            Ok(json_data) => {
                                let map: Result<Map, _> =
                                    serde_json::from_value(json_data["map"].clone());
                                let players: Result<BTreeMap<u16, Player>, _> =
                                    serde_json::from_value(json_data["players"].clone());
                                let teams: Result<BTreeMap<String, usize>, _> =
                                    serde_json::from_value(json_data["teams"].clone());
                                if let (Ok(map), Ok(players), Ok(teams)) = (map, players, teams) {
                                    if data_tx.send(ServerData::new(map, players, teams)).is_err() {
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
                }
            }
            tokio::time::sleep(Duration::from_secs(1)).await;
        }
    });

    args.engine.render(event_rx, data_rx).await
}
