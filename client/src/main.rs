use clap::Parser;
use std::{
    io::{Read as _, Write},
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

    let mut buffer = [0; 1024];
    let bytes_read = stream.read(&mut buffer).unwrap();
    let response = String::from_utf8_lossy(&buffer[..bytes_read]);
    println!("Server response: {}", response);
    if response != "BIENVENUE\n" {
        return Err("Server did not greet (Maybe server is not a zappy server)".into());
    }

    // TODO validate team
    stream.write_all(args.team.as_bytes())?;
    let bytes_read = stream.read(&mut buffer).unwrap();
    let response = String::from_utf8_lossy(&buffer[..bytes_read]);
    println!("Server response: {}", response);

    // TODO ai

    Ok(())
}
