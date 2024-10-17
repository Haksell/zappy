mod args;
mod client_connection;
mod game_loop;
mod logger;
mod map;
mod player;
mod regular;
mod server;
mod stream;

use crate::args::ServerArgs;
use crate::game_loop::game_loop;
use crate::logger::init_logger;
use crate::server::Server;
use clap::Parser;
use regular::handle_regular_connection;
use shared::GFX_PORT;
use std::error::Error;
use std::sync::Arc;
use std::time::Duration;
use stream::handle_stream_connection;
use tokio::net::TcpListener;
use tokio::sync::Mutex;
use tokio::time::interval;

const HANDSHAKE_MSG: &'static str = "BIENVENUE\n";

#[derive(Debug)]
pub enum ZappyError {
    ConnectionClosedByClient,
    MaxPlayersReached,
    ConnectionCorrupted,
    AlreadyConnected,
    TryToDisconnectNotConnected,
    TeamDoesntExist,
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
    let args = ServerArgs::parse();
    let port = args.port;
    let (server, regular_listener) = Server::from(&args).await?;
    let graphic_listener = TcpListener::bind(format!("127.0.0.1:{GFX_PORT}")).await?;
    let server = Arc::new(Mutex::new(server));

    log::debug!("Server running on 127.0.0.1:{port} (regular) and 127.0.0.1:{GFX_PORT} (stream)");

    tokio::select! {
        _ = handle_regular_connection(Arc::clone(&server), regular_listener) => {},
        _ = handle_stream_connection(Arc::clone(&server), graphic_listener) => {},
        _ = game_loop(Arc::clone(&server), args.tud) => {},
    };

    Ok(())
}
