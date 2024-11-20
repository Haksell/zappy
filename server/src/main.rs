mod args;
mod client_connection;
mod client_loop;
mod game_loop;
mod gfx_loop;
mod logger;
mod server;

use crate::args::ServerArgs;
use crate::game_loop::game_loop;
use crate::logger::init_logger;
use crate::server::Server;
use clap::Parser;
use client_loop::client_loop;
use gfx_loop::gfx_loop;
use shared::{ServerCommandToClient, GFX_PORT};
use std::collections::HashMap;
use std::error::Error;
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::sync::mpsc::Sender;
use tokio::sync::Mutex;

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn Error>> {
    init_logger();
    let args = ServerArgs::parse();
    let server = Server::from(&args).await?;
    let client_listener = TcpListener::bind(format!("127.0.0.1:{}", args.port)).await?;
    let gfx_listener = TcpListener::bind(format!("127.0.0.1:{}", GFX_PORT)).await?;
    let client_connections: Arc<Mutex<HashMap<u16, Sender<ServerCommandToClient>>>> =
        Arc::new(Mutex::new(HashMap::new()));
    let server = Arc::new(Mutex::new(server));

    log::info!(
        "Server running on 127.0.0.1:{} (client) and 127.0.0.1:{GFX_PORT} (gfx)",
        args.port
    );

    tokio::select! {
        _ = client_loop(Arc::clone(&server), Arc::clone(&client_connections), client_listener) => {},
        _ = gfx_loop(Arc::clone(&server), gfx_listener) => {},
        _ = game_loop(server, client_connections, args.tud) => {},
    };

    Ok(())
}
