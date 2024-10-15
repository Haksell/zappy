use clap::Parser;
mod client;
mod server;

use crate::client::Client;
use crate::server::Server;
use chrono::Local;
use colored::Colorize;
use env_logger::Builder;
use log::{Level, LevelFilter};
use std::error::Error;
use std::io::Write;
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::sync::{mpsc, Mutex};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long, help = "Port number")]
    port: u16,

    #[arg(short('x'), long, help = "World width")]
    width: u16,

    #[arg(short('y'), long, help = "World height")]
    height: u16,

    #[arg(
        short,
        long,
        help = "Number of clients authorized at the beginning of the game"
    )]
    clients: u16,

    #[arg(
        short,
        long,
        help = "Time Unit Divider (the greater t is, the faster the game will go)"
    )]
    tud: u16,

    #[arg(short, long, help = "List of team names", required = true, num_args = 1..)]
    names: Vec<String>,
}

const WIDTH: u32 = 30;
const HEIGHT: u32 = 20;
const HANDSHAKE_MSG: &'static str = "BIENVENUE\n";

//TODO: change to info the default log level
fn init_logger() {
    Builder::new()
        .filter(None, LevelFilter::Debug)
        .format(|buf, record| {
            let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S");
            let level = match record.level() {
                Level::Error => "ERROR".red().bold(),
                Level::Warn => "WARN".yellow().bold(),
                Level::Info => "INFO".green().bold(),
                Level::Debug => "DEBUG".blue().bold(),
                Level::Trace => "TRACE".magenta().bold(),
            };
            writeln!(
                buf,
                "{} [{}]: {}",
                timestamp,
                level,
                record.args().to_string().trim_end()
            )
        })
        .init();
    log::debug!("Starting the server");
}

#[derive(Debug)]
pub enum ZappyError {
    TechnicalError(String),
    LogicalError(String),
}

enum ServerCommandToClient {
    Shutdown,
    SendMessage(String),
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn Error>> {
    init_logger();
    let args = Args::parse();
    let addr = format!("127.0.0.1:{}", args.port);
    let server = Arc::new(Mutex::new(Server::new(1))); // TODO: args.clients?

    let listener = TcpListener::bind(&addr).await?;
    log::debug!("Listening on: {}", addr);

    loop {
        let (socket, addr) = listener.accept().await?;
        log::debug!("New connection from: {}", addr);
        let mut client = Client::new(socket, addr.clone());
        let server = Arc::clone(&server);

        tokio::spawn(async move {
            let bidon: Result<(), ZappyError> = async {
                //TODO: review the queue size
                let (cmd_tx, cmd_rx) = mpsc::channel::<ServerCommandToClient>(32);
                client.write_socket(HANDSHAKE_MSG).await?;
                let team_name = client.read_socket().await?;
                if !server.lock().await.add_client(team_name, addr, cmd_tx) {
                    client.write_socket("Too many clients\n").await?;
                };
                client
                    .write_socket(&format!("{}\n", server.lock().await.remaining_clients()))
                    .await?;
                client
                    .write_socket(&format!("{} {}\n", WIDTH, HEIGHT))
                    .await?;

                loop {
                    let s = client.read_socket().await?;
                    log::debug!("{:?} {}", client.get_addr(), s);

                    client.write_socket(&s).await?;
                }
            }
            .await;

            if let Err(err) = bidon {
                server.lock().await.remove_client(client.get_addr());
                log::error!("{:?}", err);
            }
        });
    }
}

/*
async fn handle_client(mut client: Client, mut cmd_rx: mpsc::Receiver<ServerCommandToClient>) -> std::io::Result<()> {
    let mut buf = [0u8; 1024];

    loop {
        tokio::select! {
            result = client.read(&mut buf) => {
                let n = result?;
                if n == 0 {
                    println!("Client disconnected");
                    return Ok(());
                }
                client.write_all(&buf[..n]).await?;
            }

            // Handle commands from the server
            Some(cmd) = cmd_rx.recv() => {
                match cmd {
                    ServerCommandToClient::Shutdown => {
                        println!("Shutdown command received. Closing connection.");
                        let goodbye = b"Server is shutting down the connection.\n";
                        client.write_all(goodbye).await?;
                        return Ok(());
                    }
                    ServerCommandToClient::PrintMessage(message) => {
                        println!("Sending message to client: {}", message.trim_end());
                        client.write_all(message.as_bytes()).await?;
                    }
                }
            }

            else => {
                return Ok(());
            }
        }
    }
}


 */
