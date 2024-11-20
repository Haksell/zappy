use crate::server::Server;
use serde_json::{json, to_string};
use std::collections::HashMap;
use std::error::Error;
use std::hash::{DefaultHasher, Hash, Hasher};
use std::io::IoSlice;
use std::sync::Arc;
use std::time::Duration;
use tokio::io::AsyncWriteExt;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::Mutex;
use tokio::time::sleep;

pub async fn gfx_loop(
    server: Arc<Mutex<Server>>,
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
    server: Arc<Mutex<Server>>,
    mut socket: TcpStream,
) -> std::io::Result<()> {
    let mut last_hash = 0;

    loop {
        sleep(Duration::from_millis(1000)).await;
        let (json_data, new_hash) = {
            let mut hasher = DefaultHasher::new();
            let server_lock = server.lock().await;
            server_lock.hash(&mut hasher);
            let new_hash = hasher.finish();
            if new_hash == last_hash {
                continue;
            }
            let state = json!({
                "teams": server_lock.teams().iter()
                    .map(|(k, v)| (k.clone(), v.len()))
                    .collect::<HashMap<String, usize>>(),
                "map": server_lock.map(),
                "players": server_lock.players()
            });

            (to_string(&state)?, new_hash)
        };

        last_hash = new_hash;

        socket
            .write_vectored(&[IoSlice::new(json_data.as_bytes()), IoSlice::new(b"\n")])
            .await?;
    }
}
