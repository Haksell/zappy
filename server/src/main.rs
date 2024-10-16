mod args;
mod client_connection;
mod logger;
mod player;
mod server;

use crate::args::ServerArgs;
use crate::client_connection::ClientConnection;
use crate::logger::init_logger;
use crate::server::Server;
use clap::Parser;
use std::error::Error;
use std::sync::Arc;
use tokio::sync::{mpsc, Mutex};

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
    let (server, listener) = Server::from(args).await?;
    let server = Arc::new(Mutex::new(server));

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

                return handle_client(&mut client, cmd_rx).await;
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

async fn handle_client(
    client: &mut ClientConnection,
    mut cmd_rx: mpsc::Receiver<ServerCommandToClient>,
) -> Result<(), ZappyError> {
    loop {
        tokio::select! {
            result = client.read() => {
                let n = result?;
                log::debug!("{:?}: {:?}", client.get_addr(), n);
                client.write(&n).await?
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
