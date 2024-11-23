mod args;
mod connection;
mod game_engine;
mod logger;
mod routine;
mod security;

use crate::args::ServerArgs;
use crate::game_engine::GameEngine;
use crate::logger::init_logger;
use crate::routine::admin::admin_routine;
use crate::security::tls::setup_tls;
use clap::Parser;
use dotenv::dotenv;
use routine::client::client_routine;
use routine::game::game_routine;
use routine::gfx::gfx_routine;
use security::security_context::SecurityContext;
use shared::{ServerCommandToClient, ADMIN_PORT, GFX_PORT};
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
    init_logger();

    let credentials = env::var("CREDS").expect("CREDS must be set");
    let args = ServerArgs::parse();
    let server = GameEngine::from(&args).await?;
    let client_listener = TcpListener::bind(format!("127.0.0.1:{}", args.port)).await?;
    let admin_listener = TcpListener::bind(format!("127.0.0.1:{}", ADMIN_PORT)).await?;
    let gfx_listener = TcpListener::bind(format!("127.0.0.1:{}", GFX_PORT)).await?;
    let security_context = Arc::new(Mutex::new(SecurityContext::new(credentials)?));
    let player_senders: Arc<Mutex<HashMap<u16, Sender<ServerCommandToClient>>>> =
        Arc::new(Mutex::new(HashMap::new()));
    let server = Arc::new(Mutex::new(server));
    let acceptor = setup_tls()?;

    log::info!(
        "Server running on 127.0.0.1:{} (client), 127.0.0.1:{} (admin), 127.0.0.1:{} (gfx)",
        args.port,
        ADMIN_PORT,
        GFX_PORT
    );

    tokio::select! {
        _ = client_routine(Arc::clone(&server), Arc::clone(&player_senders), client_listener) => {},
        _ = admin_routine(Arc::clone(&server), Arc::clone(&player_senders), (admin_listener, acceptor), Arc::clone(&security_context)) => {},
        _ = gfx_routine(Arc::clone(&server), gfx_listener) => {},
        _ = game_routine(server, Arc::clone(&player_senders), args.tud) => {},
    };

    Ok(())
}
