use crate::server::Server;
use serde_json::to_string;
use shared::{Cell,Egg,Resource};
use std::error::Error;
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
        log::debug!("New streaming client connected: {}", addr);
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
    loop {
        /************************************************/
        let mut cell = Cell::new();
        cell.players = vec![String::from("P1")];
        cell.resources.insert(Resource::Deraumere, 2);
        cell.resources.insert(Resource::Nourriture, 2);
        cell.eggs.push(Egg {
            start_frame: 0,
            team_name: String::from("Axel"),
        });
        server.lock().await.map.map[0][0] = cell;
        /************************************************/

        let json_data = to_string(&server.lock().await.map)?;

        // TODO: don't send if no changes (dirty state or hash)
        socket.write_all(json_data.as_bytes()).await?;
        socket.write_all(b"\n").await?;

        sleep(Duration::from_millis(1000)).await;
    }
}
