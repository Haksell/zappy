use crate::ZappyError;
use std::net::SocketAddr;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

const BUF_SIZE: usize = 1024;

pub type VoidResult<T> = Result<(), T>;

pub struct ClientConnection {
    tcp_stream: TcpStream,
    buf: Vec<u8>,
    addr: SocketAddr,
}

impl ClientConnection {
    //TODO: check when msg > buf size
    pub fn new(tcp_stream: TcpStream, addr: SocketAddr) -> Self {
        let buf = vec![0u8; BUF_SIZE];
        Self {
            tcp_stream,
            buf,
            addr,
        }
    }

    pub async fn writeln(&mut self, message: &str) -> VoidResult<ZappyError> {
        let mut buffer = Vec::with_capacity(message.len() + 1);
        buffer.extend_from_slice(message.as_bytes());
        buffer.push(b'\n');

        let message_with_newline = String::from_utf8(buffer)
            .map_err(|e| ZappyError::TechnicalError(format!("Invalid UTF-8 sequence: {}", e)))?;

        self.write(&message_with_newline).await
    }

    pub async fn write(&mut self, message: &str) -> VoidResult<ZappyError> {
        Ok(self
            .tcp_stream
            .write_all(message.as_bytes())
            .await
            .map_err(|e| ZappyError::TechnicalError(format!("Failed to write to socket: {}", e)))?)
    }

    // TODO: handle multiline commands and buffer with Ctrl+D like ft_irc/webserv
    pub async fn read(&mut self) -> Result<String, ZappyError> {
        let n = self.tcp_stream.read(&mut self.buf).await.map_err(|e| {
            ZappyError::TechnicalError(format!("Failed to read data from socket: {}", e))
        })?;
        if n == 0 {
            Err(ZappyError::ConnectionClosedByClient)
        } else if n > BUF_SIZE {
            //TODO: handle properly
            Err(ZappyError::TechnicalError(
                "The buffer is too small.".to_string(),
            ))
        } else {
            Ok(String::from_utf8(self.buf[..n].to_vec()).map_err(|e| {
                ZappyError::TechnicalError(format!("Can map read data to string: {}", e))
            })?)
        }
    }

    pub fn get_addr(&self) -> &SocketAddr {
        &self.addr
    }
}
