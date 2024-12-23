mod console;
mod torus;

use clap::Parser;
use clap::ValueEnum;
use serde_json::from_str;
use shared::{GFXData, GFX_PORT};
use std::error::Error;
use std::fmt::Debug;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::net::TcpStream;
use tokio::sync::mpsc;
use tokio::sync::mpsc::UnboundedReceiver;
use tokio::time::Duration;

enum Message {
    Disconnect(Box<dyn Error + Send>),
    State(GFXData),
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

    tokio::spawn(async move {
        loop {
            match TcpStream::connect(format!("{}:{}", args.address, args.port)).await {
                Ok(stream) => {
                    eprintln!("Connected to server");
                    let reader = BufReader::new(stream);
                    let mut lines = reader.lines();

                    while let Ok(Some(line)) = lines.next_line().await {
                        match from_str::<GFXData>(&line) {
                            Ok(new_state) => match data_tx.send(Message::State(new_state)) {
                                Err(se) => {
                                    eprintln!("Send error {}.", se);
                                    break;
                                }
                                _ => {}
                            },
                            Err(e) => eprintln!("Failed to deserialize JSON: {}", e),
                        }
                    }
                    eprintln!("Connection lost, retrying...");
                }
                Err(e) => {
                    match data_tx.send(Message::Disconnect(Box::new(e))) {
                        Err(se) => eprintln!("Send error {}.", se),
                        _ => {}
                    }
                    tokio::time::sleep(Duration::from_secs(1)).await;
                }
            }
        }
    });

    args.engine.render(data_rx).await
}
