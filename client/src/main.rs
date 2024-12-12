use clap::Parser;
use shared::HANDSHAKE_MSG;
use std::{
    io::{BufRead as _, BufReader, Read as _, Write},
    net::TcpStream,
};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short('n'), long, help = "Team name")]
    team: String,

    #[arg(short, long, default_value_t = 8080, help = "Port of the server.")]
    port: u16,

    #[arg(short, long, default_value_t = String::from("127.0.0.1"), help = "Address of the server.")]
    address: String,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    let mut stream = TcpStream::connect(format!("{}:{}", args.address, args.port))?;
    eprintln!("Connected to server");

    let reader = BufReader::new(stream);
    let mut lines = reader.lines();

    let handshake = lines.next().expect("Handshake not found")?;
    if handshake + "\n" != HANDSHAKE_MSG {
        return Err("Invalid handshake (Server may not be a zappy server)".into());
    }

    // stream.write("".as_bytes());

    while let Some(Ok(line)) = lines.next() {
        println!("Received line '{}'", line);
    }
    eprintln!("Connection lost.");

    // TODO validate team
    // TODO ai

    Ok(())
}
