use clap::Parser;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short('n'), long, help = "Team name")]
    team: String,

    #[arg(short, long, help = "Port number")]
    port: u16,

    #[arg(short, long, default_value = "localhost")]
    host: String,
}

fn main() {
    let args = Args::parse();
    println!("{:?}", args);
}
