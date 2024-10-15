use clap::Parser;
use std::error::Error;
use std::os::fd::AsFd;
use std::sync::{Arc, Mutex};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long, help = "Port number")]
    port: u16,

    #[arg(short('x'), long, help = "World width")]
    width: u16,

    #[arg(short('y'), long, help = "World height")]
    height: u16,

    #[arg(
        short,
        long,
        help = "Number of clients authorized at the beginning of the game"
    )]
    clients: u16,

    #[arg(
        short,
        long,
        help = "Time Unit Divider (the greater t is, the faster the game will go)"
    )]
    tud: u16,

    #[arg(short, long, help = "List of team names", required = true, num_args = 1..)]
    names: Vec<String>,
}

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

    pub fn add_client(&mut self, client: &TcpStream) -> bool {
        if self.remaining_clients() == 0 {
            false
        } else {
            self.clients.push(format!("{:?}", client.as_fd()));
            true
        }
    }

    pub fn remaining_clients(&self) -> u32 {
        self.max_clients - self.clients.len() as u32
    }
}

async fn write_socket(socket: &mut TcpStream, message: &str) {
    socket
        .write_all(message.as_bytes())
        .await
        .expect("failed to write data to socket");
}

async fn read_socket(socket: &mut TcpStream) -> Option<String> {
    let mut buf = vec![0; 1024];
    let n = socket
        .read(&mut buf)
        .await
        .expect("failed to read data from socket");
    if n == 0 {
        None
    } else {
        Some(String::from_utf8(buf).unwrap())
    }
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();
    let addr = format!("127.0.0.1:{}", args.port);
    let server = Arc::new(Mutex::new(Server::new(1))); // TODO: args.clients?

    let listener = TcpListener::bind(&addr).await?;
    println!("Listening on: {}", addr);

    loop {
        let (mut socket, _) = listener.accept().await?;
        let server = Arc::clone(&server);

        tokio::spawn(async move {
            if !server.lock().unwrap().add_client(&socket) {
                write_socket(&mut socket, "Too many clients\n").await;
                return;
            };

            write_socket(&mut socket, "BIENVENUE\n").await;
            let team_name = read_socket(&mut socket).await.unwrap();
            write_socket(
                &mut socket,
                &format!("{}\n", server.lock().unwrap().remaining_clients()),
            )
            .await;
            write_socket(&mut socket, &format!("{} {}\n", args.width, args.height)).await;

            println!("Client connected in team: {team_name}");

            loop {
                let s = read_socket(&mut socket).await.unwrap();
                print!("{:?} {}", socket.as_fd(), s);
                write_socket(&mut socket, &s).await;
            }
        });
    }
}
