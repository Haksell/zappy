mod args;
mod client_connection;
mod logger;
mod map;
mod player;
mod server;

use crate::args::ServerArgs;
use crate::client_connection::ClientConnection;
use crate::logger::init_logger;
use crate::map::Play;
use crate::server::Server;
use clap::Parser;
use serde_json::{from_str, to_string};
use shared::Command;
use std::error::Error;
use std::sync::Arc;
use std::time::Duration;
use tokio::io::AsyncWriteExt;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{mpsc, Mutex};
use tokio::time::sleep;

const HANDSHAKE_MSG: &'static str = "BIENVENUE\n";

#[derive(Debug)]
pub enum ZappyError {
    ConnectionClosedByClient,
    MaxPlayersReached,
    ConnectionCorrupted,
    AlreadyConnected,
    TryToDisconnectNotConnected,
    TeamDoesntExist,
    TechnicalError(String),
    LogicalError(String),
}

enum ServerCommandToClient {
    Shutdown,
    SendMessage(String),
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn Error>> {
    init_logger();
    let args = ServerArgs::parse();
    let port = args.port;
    let (server, regular_listener) = Server::from(args).await?;
    let graphic_listener = TcpListener::bind("127.0.0.1:4242").await?;
    let server = Arc::new(Mutex::new(server));

    log::debug!("Server running on 127.0.0.1:{port} (regular) and 127.0.0.1:4242 (stream)");

    tokio::select! {
        _ = handle_regular_connection(Arc::clone(&server), regular_listener) => {},
        _ = handle_stream_connection(server, graphic_listener) => {},
    }
    Ok(())
}

async fn handle_regular_connection(
    server: Arc<Mutex<Server>>,
    listener: TcpListener,
) -> Result<(), Box<dyn Error>> {
    loop {
        let (socket, addr) = listener.accept().await?;
        log::debug!("New connection from: {}", addr);
        let mut client = ClientConnection::new(socket, addr.clone());
        let server = Arc::clone(&server);

        tokio::spawn(async move {
            let bidon: Result<(), ZappyError> = async {
                //TODO: review the queue size
                let (cmd_tx, cmd_rx) = mpsc::channel::<ServerCommandToClient>(32);
                client.write(HANDSHAKE_MSG).await?;
                let team_name = client.read().await?;
                let (width, height, remaining_clients) = {
                    let mut server_lock = server.lock().await;
                    server_lock
                        .add_player(addr, cmd_tx.clone(), team_name)
                        .await?;
                    (
                        server_lock.width,
                        server_lock.height,
                        server_lock.remaining_clients(),
                    )
                };
                client.writeln(&remaining_clients.to_string()).await?;
                client.writeln(&format!("{} {}", width, height)).await?;

                return handle_player(&mut client, cmd_rx).await;
            }
            .await;

            let _ = server.lock().await.remove_player(client.get_addr()).await;
            if let Err(err) = bidon {
                //TODO: put log level and message to the impl error block of ZappyError
                match err {
                    ZappyError::ConnectionClosedByClient => log::debug!("Client disconnected"),
                    ZappyError::MaxPlayersReached => {
                        log::debug!("Max players reached");
                        let _ = client.writeln("Max players reached").await;
                    }
                    ZappyError::TeamDoesntExist => {
                        log::debug!("Team doesn't exist");
                        let _ = client
                            .writeln("Team doesn't exist. You are disconnected")
                            .await;
                    }
                    err => log::error!("{:?}", err),
                }
            }
        });
    }
}

async fn handle_player(
    client: &mut ClientConnection,
    mut cmd_rx: mpsc::Receiver<ServerCommandToClient>,
) -> Result<(), ZappyError> {
    loop {
        tokio::select! {
            result = client.read() => {
                let n = result?;
                let trimmed = n.trim_end();
                match from_str::<Command>(trimmed) {
                    Ok(command) => {
                        log::debug!("Received command: {:?}", command);
                        //TODO: implement
                        client.writeln("OK").await?
                    },
                    Err(_) => {
                        client.writeln(&format!("Unknown command \"{}\"", trimmed)).await?;
                    }
                }
            }

            Some(cmd) = cmd_rx.recv() => {
                match cmd {
                    ServerCommandToClient::Shutdown => {
                        log::debug!("Shutdown command received. Closing connection.");
                        let goodbye = "Server is shutting down the connection.";
                        client.writeln(goodbye).await?;
                        return Ok(());
                    }
                    ServerCommandToClient::SendMessage(message) => {
                        log::debug!("Sending message to client: {}", message.trim_end());
                        client.write(&message).await?;
                    }
                }
            }

            else => {
                return Ok(());
            }
        }
    }
}

async fn handle_stream_connection(
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
/* Tests commands
tokio::spawn(async move {
tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
let a = cmd_tx
.send(ServerCommandToClient::SendMessage(
"Shutdown soon\n".to_string(),
))
.await
.map_err(|e| ZappyError::TechnicalError(e.to_string()));
log::warn!("Shutdown test start: Client send message: {:?}", a);
tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
let a = cmd_tx
.send(ServerCommandToClient::Shutdown)
.await
.map_err(|e| ZappyError::TechnicalError(e.to_string()));
log::warn!("Shutdown test end: Client shutdown message: {:?}", a);
});
*/
