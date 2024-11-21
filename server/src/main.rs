mod args;
mod connection_manager;
mod game_engine;
mod logger;
mod routine;

use crate::args::ServerArgs;
use crate::connection_manager::ConnectionManager;
use crate::game_engine::GameEngine;
use crate::logger::init_logger;
use clap::Parser;
use dotenv::dotenv;
use routine::client::client_routine;
use routine::game::game_routine;
use routine::gfx::gfx_routine;
use shared::{ServerCommandToClient, GFX_PORT};
use std::collections::HashMap;
use std::env;
use std::error::Error;
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::sync::mpsc::Sender;
use tokio::sync::Mutex;

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn Error>> {
    dotenv().ok();
    let admin_pass = env::var("ADMIN_PASS").expect("ADMIN_PASS must be set");
    init_logger();
    let args = ServerArgs::parse();
    let server = GameEngine::from(&args).await?;
    let client_listener = TcpListener::bind(format!("127.0.0.1:{}", args.port)).await?;
    let gfx_listener = TcpListener::bind(format!("127.0.0.1:{}", GFX_PORT)).await?;
    let connection_manager = Arc::new(Mutex::new(ConnectionManager::new(admin_pass)));
    let server = Arc::new(Mutex::new(server));

    log::info!(
        "Server running on 127.0.0.1:{} (client) and 127.0.0.1:{GFX_PORT} (gfx)",
        args.port
    );

    tokio::select! {
        _ = client_routine(Arc::clone(&server), Arc::clone(&connection_manager), client_listener) => {},
        _ = gfx_routine(Arc::clone(&server), gfx_listener) => {},
        _ = game_routine(server, connection_manager, args.tud) => {},
    };

    Ok(())
}
