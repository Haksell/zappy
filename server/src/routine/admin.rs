use crate::connection::{AsyncReadWrite, Connection};
use crate::game_engine::GameEngine;
use crate::security::security_context::SecurityContext;
use shared::commands::AdminCommand;
use shared::LogicalError::WrongUsernameOrPassword;
use shared::{ServerCommandToClient, ZappyError};
use std::collections::HashMap;
use std::error::Error;
use std::pin::Pin;
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::sync::mpsc::Sender;
use tokio::sync::Mutex;
use tokio_rustls::TlsAcceptor;

pub async fn admin_routine(
    server: Arc<Mutex<GameEngine>>,
    player_senders: Arc<Mutex<HashMap<u16, Sender<ServerCommandToClient>>>>,
    (listener, acceptor): (TcpListener, TlsAcceptor),
    security_context: Arc<Mutex<SecurityContext>>,
) -> Result<(), Box<dyn Error>> {
    loop {
        let (socket, addr) = listener.accept().await?;
        let acceptor = acceptor.clone();
        let id = addr.port();
        log::info!("New connection, assigned id: {}", id);

        let server_clone = Arc::clone(&server);
        let client_senders_clone = Arc::clone(&player_senders);
        let security_context = Arc::clone(&security_context);

        tokio::spawn(async move {
            match acceptor.accept(socket).await {
                Ok(tls_stream) => {
                    let stream: Pin<Box<dyn AsyncReadWrite + Send>> = Box::pin(tls_stream);
                    let mut client = Connection::new(stream, id);
                    let handle_result: Result<(), ZappyError> = async {
                        client.writeln("Username:").await?;
                        let username = client.read().await?.trim_end().to_string();
                        client.writeln("Password:").await?;
                        let password = client.read().await?.trim_end().to_string();
                        {
                            if !security_context.lock().await.is_valid(&username, &password) {
                                return Err(ZappyError::Logical(WrongUsernameOrPassword));
                            }
                        }
                        client.writeln("Hi admin!").await?;
                        return handle_admin(server_clone, &mut client, client_senders_clone).await;
                    }
                    .await;

                    //Specific client loop ends here, cleanup before quiting async task
                    log::debug!("Admin: {} has been deleted by server", id);
                    if let Err(err) = handle_result {
                        match err {
                            ZappyError::Technical(err) => log::error!("{err}"),
                            ZappyError::Logical(err) => {
                                let msg = err.to_string();
                                //TODO: handle?
                                let _ = client.writeln(msg.as_str()).await;
                                log::info!("{}", err);
                            }
                        }
                    }
                    let _ = client.writeln("Disconnected").await;
                }
                Err(e) => {
                    log::error!("TLS handshake failed for {}: {}", id, e);
                }
            };
        });
    }
}

async fn handle_admin(
    server: Arc<Mutex<GameEngine>>,
    client: &mut Connection,
    player_senders: Arc<Mutex<HashMap<u16, Sender<ServerCommandToClient>>>>,
) -> Result<(), ZappyError> {
    loop {
        let msg = client.read().await?;
        let trimmed = msg.trim_end();
        match AdminCommand::try_from(trimmed) {
            Ok(command) => command.show_off(),
            Err(err) => {
                log::error!("{}: {}", client.id(), err);
                client.writeln(&err).await?;
            }
        }
    }
}
