use crate::connection::{AsyncReadWrite, Connection};
use crate::game_engine::GameEngine;
use shared::{commands::PlayerCommand, ServerCommandToClient, ZappyError};
use std::collections::HashMap;
use std::error::Error;
use std::pin::Pin;
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::sync::mpsc::Sender;
use tokio::sync::{mpsc, Mutex};

pub async fn client_routine(
    server: Arc<Mutex<GameEngine>>,
    client_senders: Arc<Mutex<HashMap<u16, Sender<ServerCommandToClient>>>>,
    listener: TcpListener,
) -> Result<(), Box<dyn Error>> {
    loop {
        let (socket, addr) = listener.accept().await?;
        let id = addr.port();
        log::info!("New connection, assigned id: {}", id);

        let server_clone = Arc::clone(&server);
        let server_arc_for_disconnect = Arc::clone(&server_clone);
        let client_senders_clone = Arc::clone(&client_senders);

        tokio::spawn(async move {
            let (cmd_tx, cmd_rx) = mpsc::channel::<ServerCommandToClient>(32);
            let stream: Pin<Box<dyn AsyncReadWrite + Send>> = Box::pin(socket);
            let mut client = Connection::new(stream, id);
            let handle_result: Result<(), ZappyError> = async {
                client.send_handshake().await?;
                let team_name = client.read().await?.trim_end().to_string();
                let (width, height, remaining_clients) = {
                    let mut server_lock = server_clone.lock().await;
                    let remaining_clients_count =
                        server_lock.add_player(client.id(), team_name.clone())?;
                    (
                        server_lock.map_width(),
                        server_lock.map_height(),
                        remaining_clients_count,
                    )
                };
                client.writeln(&remaining_clients.to_string()).await?;
                client.writeln(&format!("{} {}", width, height)).await?;
                client_senders_clone.lock().await.insert(id, cmd_tx);
                return handle_client(server_clone, &mut client, cmd_rx).await;
            }
            .await;

            //Specific client loop ends here, cleanup before quiting async task
            client_senders_clone.lock().await.remove(&id);
            server_arc_for_disconnect.lock().await.remove_player(&id);
            log::debug!("{} has been deleted by server", id);
            if let Err(err) = handle_result {
                match err {
                    ZappyError::Technical(err) => {
                        log::error!("{err}");
                    }
                    ZappyError::Logical(err) => {
                        let msg = err.to_string();

                        //TODO: handle?
                        let _ = client.writeln(msg.as_str()).await;
                        log::info!("{}", err);
                    }
                }
            }
            let _ = client.writeln("Disconnected").await;
        });
    }
}

async fn handle_client(
    server: Arc<Mutex<GameEngine>>,
    client: &mut Connection,
    mut cmd_rx: mpsc::Receiver<ServerCommandToClient>,
) -> Result<(), ZappyError> {
    loop {
        tokio::select! {
            result = client.read() => {
                let n = result?;
                let trimmed = n.trim_end();
                    match PlayerCommand::try_from(trimmed) {
                        Ok(command) => {
                            log::info!("{}: sends command: {:?}", client.id(), command);
                            if let Some(e)= server.lock().await.take_command(&client.id(), command)? {
                                log::info!("Player {} tried to push {} in to a full queue.", client.id(), trimmed);
                                client.writeln(&e.to_string()).await?;
                            }
                        },
                        Err(err) => {
                            log::error!("{}: {}", client.id(), err);
                            client.writeln(&err).await?;
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
                        log::debug!("Sending message to client: {}", response);
                        client.writeln(&response.to_string()).await?;
                    }
                }
            }
        }
    }
}
