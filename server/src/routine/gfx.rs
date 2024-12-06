use crate::game_engine::GameEngine;
use shared::GameState;
use std::error::Error;
use std::sync::Arc;
use std::time::Duration;
use tokio::io::AsyncWriteExt;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::Mutex;

pub async fn gfx_routine(
    server: Arc<Mutex<GameEngine>>,
    listener: TcpListener,
) -> Result<(), Box<dyn Error>> {
    loop {
        let (socket, addr) = listener.accept().await?;
        log::debug!("New gfx client connected: {}", addr);
        let server_clone = Arc::clone(&server);

        tokio::spawn(async move {
            if let Err(e) = handle_streaming_client(server_clone, socket).await {
                log::error!("Error handling streaming client {}: {:?}", addr, e);
            }
        });
    }
}

async fn handle_streaming_client(
    server: Arc<Mutex<GameEngine>>,
    mut socket: TcpStream,
) -> std::io::Result<()> {
    let mut last_state = GameState::default();

    loop {
        tokio::time::sleep(Duration::from_millis(20)).await;

        let current_state = {
            let server_lock = server.lock().await;
            GameState::new(
                server_lock.map().clone(),
                server_lock.players().clone(),
                server_lock
                    .teams()
                    .iter()
                    .map(|(k, v)| (k.clone(), (v.color(), v.members_count())))
                    .collect(),
            )
        };

        if current_state != last_state {
            let serialized_state = match serde_json::to_string(&current_state) {
                Ok(json) => json,
                Err(err) => {
                    eprintln!("Failed to serialize current state: {:?}", err);
                    continue;
                }
            };

            if let Err(err) = socket.write_all(serialized_state.as_bytes()).await {
                eprintln!("Failed to write to socket: {:?}", err);
                return Err(err);
            }

            if let Err(err) = socket.write_all(b"\n").await {
                eprintln!("Failed to write delimiter to socket: {:?}", err);
                return Err(err);
            }

            last_state = current_state;
        }
    }
}
