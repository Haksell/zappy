use crate::ZappyError;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

const BUF_SIZE: usize = 1024;

pub struct Client {
    tcp_stream: TcpStream,
    buf: Vec<u8>,
}

impl Client {
    //TODO: check when msg > buf size
    pub fn new(tcp_stream: TcpStream) -> Self {
        let buf = vec![0u8; BUF_SIZE];
        Self { tcp_stream, buf }
    }

    pub async fn write_socket(&mut self, message: &str) -> Result<(), ZappyError> {
        Ok(self
            .tcp_stream
            .write_all(message.as_bytes())
            .await
            .map_err(|e| ZappyError::TechnicalError(format!("Failed to write to socket: {}", e)))?)
    }

    pub async fn read_socket(&mut self) -> Result<String, ZappyError> {
        let n = self.tcp_stream.read(&mut self.buf).await.map_err(|e| {
            ZappyError::TechnicalError(format!("Failed to read data from socket: {}", e))
        })?;
        if n == 0 {
            Err(ZappyError::TechnicalError(
                "Client has closed the connection.".to_string(),
            ))
        } else {
            Ok(String::from_utf8(self.buf.clone()).map_err(|e| {
                ZappyError::TechnicalError(format!("Can map read data to string: {}", e))
            })?)
        }
    }
}
