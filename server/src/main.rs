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
    ConnectionClosedByClient,
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
                if !server.lock().await.add_client(team_name, addr, cmd_tx.clone()) {
                    client.write_socket("Too many clients\n").await?;
                };
                tokio::spawn(async move {
                    tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
                    let a = cmd_tx.send(ServerCommandToClient::SendMessage("Shutdown soon\n".to_string())).await.map_err(|e| ZappyError::TechnicalError(e.to_string()));
                    log::warn!("Shutdown test start: Client send message: {:?}", a);
                    tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
                    let a = cmd_tx.send(ServerCommandToClient::Shutdown).await.map_err(|e| ZappyError::TechnicalError(e.to_string()));
                    log::warn!("Shutdown test end: Client shutdown message: {:?}", a);
                });
                client
                    .write_socket(&format!("{}\n", server.lock().await.remaining_clients()))
                    .await?;
                client
                    .write_socket(&format!("{} {}\n", WIDTH, HEIGHT))
                    .await?;

                return handle_client(&mut client, cmd_rx).await;
            }
            .await;

            server.lock().await.remove_client(client.get_addr());
            if let Err(err) = bidon {
                match err {
                    ZappyError::ConnectionClosedByClient => log::debug!("Client disconnected"),
                    err => log::error!("{:?}", err),
                }
            }
        });
    }
}

async fn handle_client(client: &mut Client, mut cmd_rx: mpsc::Receiver<ServerCommandToClient>) -> Result<(), ZappyError> {
    loop {
        tokio::select! {
            //TODO await? o_O
            result = client.read_socket() => {
                let n = result?;
                log::debug!("{:?}: {:?}", client.get_addr(), n);
                client.write_socket(&n).await?
            }

            Some(cmd) = cmd_rx.recv() => {
                match cmd {
                    ServerCommandToClient::Shutdown => {
                        log::debug!("Shutdown command received. Closing connection.");
                        let goodbye = "Server is shutting down the connection.\n";
                        client.write_socket(goodbye).await?;
                        return Ok(());
                    }
                    ServerCommandToClient::SendMessage(message) => {
                        log::debug!("Sending message to client: {}", message.trim_end());
                        client.write_socket(&message).await?;
                    }
                }
            }

            else => {
                return Ok(());
            }
        }
    }
}