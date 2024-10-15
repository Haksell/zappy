use std::collections::HashMap;
use std::env;
use std::error::Error;
use std::os::fd::AsFd;
use std::sync::{Arc, Mutex};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

const WIDTH: u32 = 30;
const HEIGHT: u32 = 20;

struct Server {
    pub max_clients: u32,
    pub clients: Vec<String>,
}

impl Server {
    pub fn new(max_clients: u32) -> Self {
        Self {
            max_clients,
            clients: Vec::new(),
        }
    }

    pub fn add_client(&mut self, client: &TcpStream) {
        self.clients.push(format!("{:?}", client.as_fd()));
    }
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn Error>> {
    let addr = env::args()
        .nth(1)
        .unwrap_or_else(|| "127.0.0.1:8080".to_string());
    let server = Arc::new(Mutex::new(Server::new(1)));

    let listener = TcpListener::bind(&addr).await?;
    println!("Listening on: {}", addr);

    loop {
        let (mut socket, _) = listener.accept().await?;
        let server = Arc::clone(&server);

        tokio::spawn(async move {
            let mut buf = vec![0; 1024];
            server.lock().unwrap().add_client(&socket);

            println!("{:?}", server.lock().unwrap().clients);

            loop {
                let n = socket
                    .read(&mut buf)
                    .await
                    .expect("failed to read data from socket");

                if n == 0 {
                    return;
                }

                print!(
                    "{:?} {}",
                    socket.as_fd(),
                    String::from_utf8(buf.clone()).unwrap()
                );

                socket
                    .write_all(&buf[0..n])
                    .await
                    .expect("failed to write data to socket");
            }
        });
    }
}
