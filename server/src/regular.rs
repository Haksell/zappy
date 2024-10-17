use crate::client_connection::ClientConnection;
use crate::server::Server;
use crate::{ServerCommandToClient, ZappyError, HANDSHAKE_MSG};
use serde_json::from_str;
use shared::Command;
use std::error::Error;
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::sync::{mpsc, Mutex};

pub async fn handle_regular_connection(
    server: Arc<Mutex<Server>>,
    listener: TcpListener,
) -> Result<(), Box<dyn Error>> {
    loop {
        let (socket, addr) = listener.accept().await?;
        log::debug!("New connection from: {}", addr);
        let mut client = ClientConnection::new(socket, addr.clone());

        // FIXME
        let server = Arc::clone(&server);
        let server2 = Arc::clone(&server);

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

                return handle_player(server, &mut client, cmd_rx).await;
            }
            .await;

            let _ = server2.lock().await.remove_player(client.get_addr()).await;
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
    server: Arc<Mutex<Server>>,
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
                        server.lock().await.take_command(client.get_addr(), command);
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
        }
    }
}
