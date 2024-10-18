mod args;
mod client_connection;
mod client_loop;
mod game_loop;
mod gfx_loop;
mod logger;
mod map;
mod player;
mod server;

use crate::args::ServerArgs;
use crate::game_loop::game_loop;
use crate::logger::init_logger;
use crate::server::Server;
use clap::Parser;
use client_loop::client_loop;
use gfx_loop::gfx_loop;
use shared::GFX_PORT;
use std::error::Error;
use std::sync::Arc;
use std::time::Duration;
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

    log::debug!("Server running on 127.0.0.1:{port} (client) and 127.0.0.1:{GFX_PORT} (gfx)");

    tokio::select! {
        _ = client_loop(Arc::clone(&server), regular_listener) => {},
        _ = gfx_loop(Arc::clone(&server), graphic_listener) => {},
        _ = game_loop(Arc::clone(&server), args.tud) => {},
    };

    Ok(())
}
