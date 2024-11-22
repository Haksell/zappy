use shared::TechnicalError::{
    ConnectionClosedByClient, FailedToReadFromSocket, FailedToWriteToSocket,
    MessageCantBeMappedToFromUtf8, MessageIsTooBig,
};
use shared::ZappyError::Technical;
use shared::{ZappyError, HANDSHAKE_MSG};
use std::io::IoSlice;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

const BUF_SIZE: usize = 1024;

pub struct ClientConnection {
    tcp_stream: TcpStream,
    buf: Vec<u8>,
    id: u16,
}

impl ClientConnection {
    //TODO: check when msg > buf size
    pub async fn send_handshake(&mut self) -> Result<(), ZappyError> {
        self.writeln(HANDSHAKE_MSG).await
    }

    pub fn new(tcp_stream: TcpStream, id: u16) -> Self {
        let buf = vec![0u8; BUF_SIZE];
        Self {
            buf,
            tcp_stream,
            id,
        }
    }

    pub async fn writeln(&mut self, message: &str) -> Result<(), ZappyError> {
        self.tcp_stream
            .write_vectored(&[IoSlice::new(message.as_bytes()), IoSlice::new(b"\n")])
            .await
            .map(|_| ())
            .map_err(|e| Technical(FailedToWriteToSocket(self.id, e.to_string())))
    }

    pub async fn write(&mut self, message: &str) -> Result<(), ZappyError> {
        self.tcp_stream
            .write_all(message.as_bytes())
            .await
            .map_err(|e| Technical(FailedToWriteToSocket(self.id, e.to_string())))
    }

    // TODO: handle multiline commands and buffer with Ctrl+D like ft_irc/webserv
    pub async fn read(&mut self) -> Result<String, ZappyError> {
        let n = self
            .tcp_stream
            .read(&mut *self.buf)
            .await
            .map_err(|e| Technical(FailedToReadFromSocket(self.id, e.to_string())))?;
        if n == 0 {
            Err(Technical(ConnectionClosedByClient(self.id)))
        } else if n > BUF_SIZE {
            //TODO: handle properly
            Err(Technical(MessageIsTooBig(self.id)))
        } else {
            Ok(String::from_utf8(self.buf[..n].to_vec())
                .map_err(|e| Technical(MessageCantBeMappedToFromUtf8(self.id, e.to_string())))?)
        }
    }

    pub fn id(&self) -> u16 {
        self.id
    }
}
