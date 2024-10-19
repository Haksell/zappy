use std::collections::HashMap;
use crate::server::Server;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::mpsc::Sender;
use tokio::sync::Mutex;
use shared::{ServerCommandToClient, ServerResponse};

pub async fn game_loop(server: Arc<Mutex<Server>>, client_connections: Arc<Mutex<HashMap<u16, Sender<ServerCommandToClient>>>>,
                       tud: u16) {
    let t0 = tokio::time::Instant::now();
    let mut action_execution_results: Vec<(u16, ServerResponse)> = Vec::new();

    loop {
        let frame = {
            let mut server_lock = server.lock().await;
            server_lock.tick(&mut action_execution_results);
            server_lock.frame
        };

        //TODO: review errors here
        let client_connections_lock = client_connections.lock().await;
        while let Some((addr, response)) = action_execution_results.pop() {
            if let Some(connection) = client_connections_lock.get(&addr) {
                if let Err(e) = connection.send(ServerCommandToClient::SendMessage(response)).await {
                    log::error!("Failed to send message to client: {:?}", e);
                }
            } else {
                log::error!("Can't find the player to send the action execution result")
            }
        }

        let now = tokio::time::Instant::now();
        let target = t0 + Duration::from_nanos((1e9 * frame as f64 / tud as f64) as u64);
        if now < target {
            tokio::time::sleep(target - now).await;
        } else {
            log::warn!("Time step took too long. Finished at {now:?} instead of {target:?}");
        }
    }
}
