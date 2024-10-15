use clap::Parser;
mod client;
mod server;

use crate::client::Client;
use crate::server::Server;
use std::env;
use std::error::Error;
use std::os::fd::AsFd;
use std::sync::{Arc, Mutex};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

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
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("debug")).init();
    log::debug!("Starting the server");
}

#[derive(Debug)]
pub enum ZappyError {
    TechnicalError(String),
    LogicalError(String),
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn Error>> {
    init_logger();
    let addr = env::args()
        .nth(1)
        .unwrap_or_else(|| "127.0.0.1:8080".to_string());
    let server = Arc::new(Mutex::new(Server::new(1)));
    let args = Args::parse();
    let addr = format!("127.0.0.1:{}", args.port);
    let server = Arc::new(Mutex::new(Server::new(1))); // TODO: args.clients?

    let listener = TcpListener::bind(&addr).await?;
    log::debug!("Listening on: {}", addr);

    loop {
        let (socket, _) = listener.accept().await?;
        let mut client = Client::new(socket);
        let server = Arc::clone(&server);

        tokio::spawn(async move {
            let bidon: Result<(), ZappyError> = async {
                /*
                if !server.lock().unwrap().add_client(&client) {
                    write_socket(&mut client, "Too many clients\n").await;
                    return;
                };
                 */

                client.write_socket(HANDSHAKE_MSG).await?;
                let team_name = client.read_socket().await?;
                client
                    .write_socket(&format!("{}\n", server.lock().unwrap().remaining_clients()))
                    .await?;
                client
                    .write_socket(&format!("{} {}\n", WIDTH, HEIGHT))
                    .await?;

                log::debug!("Client connected in team: {team_name}");

                loop {
                    let s = client.read_socket().await?;
                    //log::debug!("{:?} {}", client.as_fd(), s);
                    log::debug!("{:?} {}", "someone", s);

                    client.write_socket(&s).await?;
                }
            }
            .await;

            if let Err(err) = bidon {
                println!("Bidon error: {:?}", err);
            }
        });
    }
}
