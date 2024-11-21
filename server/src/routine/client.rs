use crate::connection_manager::{ClientConnection, ClientConnectionType, ConnectionManager};
use crate::game_engine::GameEngine;
use shared::commands::AdminCommand;
use shared::{
    commands::PlayerCommand, ServerCommandToClient, ZappyError, HANDSHAKE_MSG,
};
use std::collections::HashMap;
use std::error::Error;
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::sync::mpsc::Sender;
use tokio::sync::{mpsc, Mutex};


pub async fn client_routine(
    server: Arc<Mutex<GameEngine>>,
    connection_manager: Arc<Mutex<ConnectionManager>>,
    listener: TcpListener,
) -> Result<(), Box<dyn Error>> {
    loop {
        let (socket, addr) = listener.accept().await?;
        let id = addr.port();
        log::info!("New connection, assigned id: {}", id);

        let server = Arc::clone(&server);
        let server_arc_for_disconnect = Arc::clone(&server);
        let connection_manager2= Arc::clone(&connection_manager);
        let connection_manager3= Arc::clone(&connection_manager);

        tokio::spawn(async move {
            let (cmd_tx, cmd_rx) = mpsc::channel::<ServerCommandToClient>(32);
            if let Ok(mut client) = ClientConnection::new(socket, id, connection_manager3).await {
            let handle_result: Result<(), ZappyError> = async {
                connection_manager2.lock().await.add_connection(id, cmd_tx);
                match client.get_connection_type() {
                    ClientConnectionType::Admin => {
                        client.writeln("Hi admin!").await?;
                    }
                    ClientConnectionType::Player(team) => {
                        let (width, height, remaining_clients) = {
                            let mut server_lock = server.lock().await;
                            let remaining_clients_count = server_lock.add_player(client.id(), team.clone())?;
                            (
                                server_lock.width(),
                                server_lock.height(),
                                remaining_clients_count,
                            )
                        };
                        client.writeln(&remaining_clients.to_string()).await?;
                        client.writeln(&format!("{} {}", width, height)).await?;
                    }
                }
                return handle_client(server, &mut client, cmd_rx).await;
            }
            .await;

            //Specific client loop ends here, cleanup before quiting async task
            let mut client = connection_manager2.lock().await.remove_connection(id);
            server_arc_for_disconnect
                .lock()
                .await
                .remove_player(&id);
            log::debug!("{} has been deleted by server", id);
            if let Err(err) = handle_result {
                match err {
                    ZappyError::Technical(err) => {
                        log::error!("{err}");
                    }
                    ZappyError::Logical(err) => {
                        let msg = err.to_string();
                        //TODO: handle?
                        
                        //let _ = client.writeln(msg.as_str()).await;
                        log::info!("{}", err);
                    }
                }
            }}
        });
    }
}

async fn handle_client(
    server: Arc<Mutex<GameEngine>>,
    client: &mut ClientConnection,
    mut cmd_rx: mpsc::Receiver<ServerCommandToClient>,
) -> Result<(), ZappyError> {
    loop {
        tokio::select! {
            result = client.read() => {
                let n = result?;
                let trimmed = n.trim_end();
                match client.get_connection_type() {
                    ClientConnectionType::Admin => match AdminCommand::try_from(trimmed) {
                        Ok(command) => command.show_off(),
                        Err(err) => {
                            log::error!("{}: {}", client.id(), err);
                            client.writeln(&err).await?;
                        }
                    }
                    ClientConnectionType::Player(_) => match PlayerCommand::try_from(trimmed) {
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
