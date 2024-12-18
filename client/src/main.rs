use clap::Parser;
use shared::HANDSHAKE_MSG;
use std::{
    io::{BufRead as _, BufReader, Lines, Write},
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

fn send_command(
    sender: &mut TcpStream,
    receiver: &mut Lines<BufReader<TcpStream>>,
    command: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    sender.write(command.as_bytes())?;
    sender.flush()?;
    receiver
        .next()
        .ok_or_else(|| "No response from server".into())
        .and_then(|response| response.map_err(|e| e.into()))
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    let stream = TcpStream::connect(format!("{}:{}", args.address, args.port))?;
    let mut sender = stream.try_clone()?;
    eprintln!("Connected to server");

    // TODO Blocks while no line received
    let reader = BufReader::new(stream);
    let mut lines = reader.lines();

    let handshake = lines.next().expect("Handshake not found")?;
    if handshake + "\n" != HANDSHAKE_MSG {
        return Err("Invalid handshake (Server may not be a zappy server)".into());
    };

    sender.write(args.team.as_bytes())?;

    let line = lines.next().expect("Missing line from server")?;
    let n_clients: usize = line.parse().map_err(|_| line)?;

    let dimensions = lines
        .next()
        .expect("Missing line from server")?
        .split_whitespace()
        .map(ToString::to_string)
        .collect::<Vec<String>>();
    if dimensions.len() != 2 {
        return Err("Invalid dimensions from server".into());
    }

    let width: usize = dimensions[0].parse()?;
    let height: usize = dimensions[1].parse()?;

    if n_clients == 0 {
        return Err(format!("The team '{}' is full.", args.team).into());
    }

    loop {
        eprintln!("{}", send_command(&mut sender, &mut lines, "move")?);
        eprintln!("{}", send_command(&mut sender, &mut lines, "right")?);
    }

    // eprintln!("Connection lost.");

    // TODO validate team
    // TODO ai

    // Ok(())
}
