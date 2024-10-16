use crate::server::Server;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;

pub async fn game_loop(server: Arc<Mutex<Server>>, tud: u16) {
    let t0 = tokio::time::Instant::now();

    loop {
        let frame = {
            let mut server_lock = server.lock().await;
            server_lock.tick();
            server_lock.frame
        };

        let now = tokio::time::Instant::now();
        let target = t0 + Duration::from_nanos((1e9 * frame as f64 / tud as f64) as u64);
        if now < target {
            tokio::time::sleep(target - now).await;
        } else {
            log::warn!("Time step took too long. Finished at {now:?} instead of {target:?}");
        }
    }
}
