mod console;
mod torus;

use clap::Parser;
use clap::ValueEnum;
use crossterm::event::{self, Event, KeyEvent};
use serde_json::{from_str, Value};
use shared::map::Map;
use shared::player::Player;
use shared::{ServerData, GFX_PORT};
use std::collections::BTreeMap;
use std::fmt::Debug;
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::net::TcpStream;
use tokio::sync::mpsc;
use tokio::sync::mpsc::UnboundedReceiver;
use tokio::time::Duration;

enum Message {
    Disconnect,
    KeyEvent(KeyEvent),
    Data(ServerData),
}

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

#[derive(ValueEnum, Clone, Debug, PartialEq, Eq)]
enum Engine {
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

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    let (data_tx, data_rx) = mpsc::unbounded_channel();
    let data_tx = Arc::new(data_tx);

    if args.engine == Engine::Console {
        let key_tx = Arc::clone(&data_tx);
        tokio::spawn(async move {
            loop {
                let poll = tokio::task::spawn_blocking(|| event::poll(Duration::from_millis(50)))
                    .await
                    .unwrap();

                if let Ok(true) = poll {
                    let evt = tokio::task::spawn_blocking(|| event::read()).await.unwrap();
                    if let Ok(Event::Key(key)) = evt {
                        if key_tx.send(Message::KeyEvent(key)).is_err() {
                            break;
                        }
                    }
                }
                tokio::time::sleep(Duration::from_millis(50)).await;
            }
        });
    }

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
                                    if data_tx
                                        .send(Message::Data(ServerData::new(map, players, teams)))
                                        .is_err()
                                    {
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
                    let _ = data_tx.send(Message::Disconnect);
                    eprintln!("Failed to connect: {}, retrying in 1 second...", e);
                    tokio::time::sleep(Duration::from_secs(1)).await;
                }
            }
        }
    });

    args.engine.render(data_rx).await
}
