use crate::client_connection::ClientConnection;
use crate::server::Server;
use shared::{command::Command, ServerCommandToClient, ZappyError, HANDSHAKE_MSG};
use std::collections::HashMap;
use std::error::Error;
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::sync::mpsc::Sender;
use tokio::sync::{mpsc, Mutex};

pub async fn do_handshake(
    client: &mut ClientConnection,
    server: &Arc<Mutex<Server>>,
    cmd_tx: Sender<ServerCommandToClient>,
) -> Result<(), ZappyError> {
    client.write(HANDSHAKE_MSG).await?;
    let team_name = client.read().await?;
    let (width, height, remaining_clients) = {
        let mut server_lock = server.lock().await;
        let remaining_clients_count = server_lock.add_player(client.id(), cmd_tx, team_name)?;
        (
            server_lock.width(),
            server_lock.height(),
            remaining_clients_count,
        )
    };
    client.writeln(&remaining_clients.to_string()).await?;
    client.writeln(&format!("{} {}", width, height)).await
}

pub async fn client_loop(
    server: Arc<Mutex<Server>>,
    client_connections: Arc<Mutex<HashMap<u16, Sender<ServerCommandToClient>>>>,
    listener: TcpListener,
) -> Result<(), Box<dyn Error>> {
    loop {
        let (socket, addr) = listener.accept().await?;
        log::info!("New connection, assigned id: {}", addr.port());
        let mut client = ClientConnection::new(socket, addr.port());

        let server = Arc::clone(&server);
        let server_arc_for_disconnect = Arc::clone(&server);
        let client_connections = Arc::clone(&client_connections);

        tokio::spawn(async move {
            let handle_result: Result<(), ZappyError> = async {
                let (cmd_tx, cmd_rx) = mpsc::channel::<ServerCommandToClient>(32);
                do_handshake(&mut client, &server, cmd_tx.clone()).await?;
                client_connections.lock().await.insert(client.id(), cmd_tx);
                return handle_player(server, &mut client, cmd_rx).await;
            }
            .await;

            //Specific client loop ends here, cleanup before quiting async task
            client_connections.lock().await.remove(&client.id());
            server_arc_for_disconnect
                .lock()
                .await
                .remove_player(&client.id());
            log::debug!("{} has been deleted by server", client.id());
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
                match Command::try_from(trimmed) {
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
