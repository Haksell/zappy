use crate::game_engine::GameEngine;
use shared::{ServerCommandToClient, ServerResponse};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::mpsc::Sender;
use tokio::sync::Mutex;

pub async fn game_routine(
    server: Arc<Mutex<GameEngine>>,
    client_senders: Arc<Mutex<HashMap<u16, Sender<ServerCommandToClient>>>>,
    tud: u16,
) {
    let t0 = tokio::time::Instant::now();
    let mut execution_results_buffer: Vec<(u16, ServerResponse)> = Vec::new();

    loop {
        let frame = {
            let mut server_lock = server.lock().await;
            server_lock.tick(&mut execution_results_buffer);
            *server_lock.frame()
        };

        while let Some((client_id, response)) = execution_results_buffer.pop() {
            if let Some(connection) = client_senders.lock().await.get(&client_id) {
                //TODO: investigate is "slow reader" case is possible?
                //TODO: for example can send take a lot of time to block us here?
                if let Err(e) = connection
                    .send(ServerCommandToClient::SendMessage(response))
                    .await
                {
                    log::error!("Failed to send message to client: {:?}", e);
                }
            } else {
                log::warn!("Can't find the player with id {client_id} to send the action execution result. Probably already disconnected.");
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
