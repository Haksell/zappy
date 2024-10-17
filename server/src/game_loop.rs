use crate::server::Server;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;

pub async fn game_loop(server: Arc<Mutex<Server>>, tud: u16) {
    let start_time = tokio::time::Instant::now();
    for step in 1u64.. {
        server.lock().await.tick();
        let elapsed = start_time.elapsed();
        let target = start_time + Duration::from_nanos((1e9 * step as f64 / tud as f64) as u64);
        if start_time + elapsed < target {
            tokio::time::sleep(target - (start_time + elapsed)).await;
        }
    }
}
