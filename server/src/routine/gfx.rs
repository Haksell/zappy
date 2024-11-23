use crate::game_engine::GameEngine;
use serde_json::json;
use std::collections::HashMap;
use std::error::Error;
use std::io::IoSlice;
use std::sync::Arc;
use std::time::Duration;
use tokio::io::AsyncWriteExt;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::Mutex;
use tokio::time::sleep;

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
    let mut last_state = json!({});

    loop {
        sleep(Duration::from_millis(20)).await;

        let current_state = {
            let server_lock = server.lock().await;
            json!({
                "teams": server_lock.teams().iter()
                    .map(|(k, v)| (k.clone(), v.len()))
                    .collect::<HashMap<String, usize>>(),
                "map": server_lock.map(),
                "players": server_lock.players()
            })
        };

        if current_state != last_state {
            println!("Updating the state... //TODO: delete, it is still here to manually check if it works fine :P");
            socket
                .write_vectored(&[
                    IoSlice::new(current_state.to_string().as_bytes()),
                    IoSlice::new(b"\n"),
                ])
                .await?;
            last_state = current_state;
        }
    }
}
