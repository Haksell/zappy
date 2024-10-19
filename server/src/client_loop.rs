use std::collections::HashMap;
use crate::client_connection::ClientConnection;
use crate::server::Server;
use serde_json::from_str;
use shared::{Command, ServerCommandToClient, ZappyError, HANDSHAKE_MSG};
use std::error::Error;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::sync::{mpsc, Mutex};
use tokio::sync::mpsc::Sender;

pub async fn client_loop(
    server: Arc<Mutex<Server>>,
    client_connections: Arc<Mutex<HashMap<SocketAddr, Sender<ServerCommandToClient>>>>,
    listener: TcpListener,
) -> Result<(), Box<dyn Error>> {
    loop {
        let (socket, addr) = listener.accept().await?;
        log::debug!("New connection from: {}", addr);
        let mut client = ClientConnection::new(socket, addr.clone());

        // FIXME
        let server = Arc::clone(&server);
        let server2 = Arc::clone(&server);
        let client_connections = Arc::clone(&client_connections);

        tokio::spawn(async move {

            let bidon: Result<(), ZappyError> = async {
                //TODO: review the queue size
                let (cmd_tx, cmd_rx) = mpsc::channel::<ServerCommandToClient>(32);
                client.write(HANDSHAKE_MSG).await?;
                let team_name = client.read().await?;
                let (width, height, remaining_clients) = {
                    let mut server_lock = server.lock().await;
                    server_lock
                        .add_player(addr, cmd_tx.clone(), team_name)?;
                    (
                        server_lock.width,
                        server_lock.height,
                        server_lock.remaining_clients(),
                    )
                };
                client_connections.lock().await.insert(addr, cmd_tx);
                client.writeln(&remaining_clients.to_string()).await?;
                client.writeln(&format!("{} {}", width, height)).await?;

                return handle_player(server, &mut client, cmd_rx).await;
            }
            .await;
            
            //Specific client loop ends here, cleanup before quiting async task

            client_connections.lock().await.remove(&addr);
            //TODO: handle
            let _ = server2.lock().await.remove_player(client.get_addr());
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
                        if let Err(e)= server.lock().await.take_command(client.get_addr(), command) {
                            match e {
                                ZappyError::Waring(msg) => client.writeln(msg.get_text()).await?,
                                _ => return Err(e),
                            }
                            //TODO: test lock here
                        }
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
                    ServerCommandToClient::SendMessage(response) => {
                        log::debug!("Sending message to client: {}", response.get_text());
                        client.writeln(response.get_text()).await?;
                    }
                }
            }
        }
    }
}
