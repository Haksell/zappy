use shared::NetworkError::{
    ConnectionClosedByClient, FailedToReadFromSocket, FailedToWriteToSocket,
    MessageCantBeMappedToFromUtf8, MessageIsTooBig,
};
use shared::ZappyError::Network;
use shared::{ZappyError, HANDSHAKE_MSG};
use std::pin::Pin;
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};

const BUF_SIZE: usize = 1024;

pub trait AsyncReadWrite: AsyncRead + AsyncWrite {}
impl<T: AsyncRead + AsyncWrite + ?Sized> AsyncReadWrite for T {}

pub struct Connection {
    stream: Pin<Box<dyn AsyncReadWrite + Send>>,
    buf: Vec<u8>,
    id: u16,
}

impl Connection {
    //TODO: check when msg > buf size
    pub async fn send_handshake(&mut self) -> Result<(), ZappyError> {
        self.write(HANDSHAKE_MSG).await
    }

    pub fn new(stream: Pin<Box<dyn AsyncReadWrite + Send>>, id: u16) -> Self {
        let buf = vec![0u8; BUF_SIZE];
        Self { buf, stream, id }
    }

    pub async fn writeln(&mut self, message: &str) -> Result<(), ZappyError> {
        self.stream
            .write_all(format!("{}\n", message).as_bytes())
            .await
            .map_err(|e| Network(FailedToWriteToSocket(self.id, e.to_string())))
    }

    pub async fn write(&mut self, message: &str) -> Result<(), ZappyError> {
        self.stream
            .write_all(message.as_bytes())
            .await
            .map_err(|e| Network(FailedToWriteToSocket(self.id, e.to_string())))
    }

    // TODO: handle multiline commands and buffer with Ctrl+D like ft_irc/webserv
    pub async fn read(&mut self) -> Result<String, ZappyError> {
        let n = self
            .stream
            .read(&mut *self.buf)
            .await
            .map_err(|e| Network(FailedToReadFromSocket(self.id, e.to_string())))?;
        if n == 0 {
            Err(Network(ConnectionClosedByClient(self.id)))
        } else if n > BUF_SIZE {
            //TODO: handle properly
            Err(Network(MessageIsTooBig(self.id)))
        } else {
            Ok(String::from_utf8(self.buf[..n].to_vec())
                .map_err(|e| Network(MessageCantBeMappedToFromUtf8(self.id, e.to_string())))?)
        }
    }

    pub fn id(&self) -> u16 {
        self.id
    }
}
