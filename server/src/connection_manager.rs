use shared::TechnicalError::{
    ConnectionClosedByClient, FailedToReadFromSocket, FailedToWriteToSocket,
    MessageCantBeMappedToFromUtf8, MessageIsTooBig,
};
use shared::ZappyError::Technical;
use shared::{ServerCommandToClient, ZappyError, HANDSHAKE_MSG};
use std::collections::HashMap;
use std::hash::{DefaultHasher, Hash, Hasher};
use std::io::IoSlice;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::sync::mpsc::Sender;
use tokio::sync::Mutex;

const BUF_SIZE: usize = 1024;
pub const ADMIN_MESSAGE: &'static str = "admin42";

pub struct ConnectionManager {
    connections: HashMap<u16, Sender<ServerCommandToClient>>,
    admin_password: u64,
}

impl ConnectionManager {
    pub fn new(mut admin_password: String) -> ConnectionManager {
        let mut s = DefaultHasher::new();
        admin_password.hash(&mut s);
        let hashed = s.finish();
        admin_password.clear();
        Self {
            connections: HashMap::new(),
            admin_password: hashed,
        }
    }

    async fn handle_admin_connection(
        client_connection: &mut ClientConnection,
        connection_manager: Arc<Mutex<ConnectionManager>>,
    ) -> Result<(), ZappyError> {
        //TODO: auth here
        let id = client_connection.id;
        Ok(())
    }


    pub fn add_connection(&mut self, id:u16, connection: Sender<ServerCommandToClient>)  {
        self.connections.insert(   id, connection);
    }

    pub fn remove_connection(&mut self, id: u16) -> Sender<ServerCommandToClient> {
        self.connections.remove(&id).unwrap()
    }

    pub fn get_connection(&mut self, id: &u16) -> Option<&mut Sender<ServerCommandToClient>> {
        self.connections.get_mut(&id)
    }
}

pub enum ClientConnectionType {
    Admin,
    Player(String),
}

pub struct ClientConnection {
    connection_type: ClientConnectionType,
    tcp_stream: TcpStream,
    buf: Vec<u8>,
    id: u16,
}

impl ClientConnection {
    //TODO: check when msg > buf size
    async fn send_handshake(socket: &mut TcpStream, id: u16) -> Result<(), ZappyError> {
        Self::generic_writeln(socket, id, HANDSHAKE_MSG).await
    }

    pub async fn new(
        mut tcp_stream: TcpStream,
        id: u16,
        connection_manager: Arc<Mutex<ConnectionManager>>,
    ) -> Result<Self, ZappyError> {
        let mut buf = vec![0u8; BUF_SIZE];
        Self::send_handshake(&mut tcp_stream, id).await?;
        let client_first_msg = Self::generic_read( &mut tcp_stream, id, &mut buf).await?.trim().to_string();
        Ok(if client_first_msg == ADMIN_MESSAGE {
            //TODO: auth
            Self {
                buf,
                tcp_stream,
                id,
                connection_type: ClientConnectionType::Admin
            }
        } else {
            Self {
                buf,
                tcp_stream,
                id,
                connection_type: ClientConnectionType::Admin
            }
        })
    }

    pub async fn writeln(&mut self, message: &str) -> Result<(), ZappyError> {
        Self::generic_writeln(&mut self.tcp_stream, self.id, message).await
    }

    pub async fn write(&mut self, message: &str) -> Result<(), ZappyError> {
        Self::generic_write(&mut self.tcp_stream, self.id, message).await
    }

    pub async fn read(&mut self) -> Result<String, ZappyError> {
        Self::generic_read(&mut self.tcp_stream, self.id, &mut self.buf).await
    }

    async fn generic_writeln(
        socket: &mut TcpStream,
        id: u16,
        message: &str,
    ) -> Result<(), ZappyError> {
        socket
            .write_vectored(&[IoSlice::new(message.as_bytes()), IoSlice::new(b"\n")])
            .await
            .map(|_| ())
            .map_err(|e| Technical(FailedToWriteToSocket(id, e.to_string())))
    }

    async fn generic_write(
        socket: &mut TcpStream,
        id: u16,
        message: &str,
    ) -> Result<(), ZappyError> {
        socket
            .write_all(message.as_bytes())
            .await
            .map_err(|e| Technical(FailedToWriteToSocket(id, e.to_string())))
    }

    // TODO: handle multiline commands and buffer with Ctrl+D like ft_irc/webserv
    async fn generic_read(
        socket: &mut TcpStream,
        id: u16,
        buf: &mut Vec<u8>,
    ) -> Result<String, ZappyError> {
        let n = socket
            .read(buf)
            .await
            .map_err(|e| Technical(FailedToReadFromSocket(id, e.to_string())))?;
        if n == 0 {
            Err(Technical(ConnectionClosedByClient(id)))
        } else if n > BUF_SIZE {
            //TODO: handle properly
            Err(Technical(MessageIsTooBig(id)))
        } else {
            Ok(String::from_utf8(buf[..n].to_vec())
                .map_err(|e| Technical(MessageCantBeMappedToFromUtf8(id, e.to_string())))?)
        }
    }

    pub fn id(&self) -> u16 {
        self.id
    }

    pub fn get_connection_type(&self) -> &ClientConnectionType {
        &self.connection_type
    }

}
