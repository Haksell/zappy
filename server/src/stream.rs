use crate::map::Play;
use crate::server::Server;
use serde_json::to_string;
use std::error::Error;
use std::sync::Arc;
use std::time::Duration;
use tokio::io::AsyncWriteExt;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::Mutex;
use tokio::time::sleep;

pub async fn handle_stream_connection(
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
        let mut server_lock = server.lock().await;
        let json_data = to_string(&server_lock.map)?;
        server_lock.map.next_position();

        socket.write_all(json_data.as_bytes()).await?;
        socket.write_all(b"\n").await?; // Add a newline for easier reading

        sleep(Duration::from_secs(1)).await;
    }
}
