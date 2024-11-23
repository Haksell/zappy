use clap::Parser;
use shared::MAX_PLAYERS_IN_TEAM;

// TODO: more default values
// TODO: min max value for width and height

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub(crate) struct ServerArgs {
    #[arg(short, long, help = "Port number", default_value_t = 8080)]
    pub(crate) port: u16,

    #[arg(short('x'), long, help = "World width")]
    pub(crate) width: usize,

    #[arg(short('y'), long, help = "World height")]
    pub(crate) height: usize,

    #[arg(
        short,
        long,
        default_value_t = 1,
        value_parser = clients_in_range,
        help = "Number of clients authorized at the beginning of the game")]
    pub(crate) clients: u16,

    #[arg(
        short,
        long,
        help = "Time Unit Divider (the greater t is, the faster the game will go)",
        default_value_t = 100
    )]
    pub(crate) tud: u16,

    #[arg(short, long, help = "List of team names", required = true, num_args = 1..)]
    pub(crate) names: Vec<String>,
}

fn clients_in_range(s: &str) -> Result<u16, String> {
    let clients: u16 = s.parse().map_err(|_| "Not a valid number")?;
    if clients > 0 && clients <= MAX_PLAYERS_IN_TEAM {
        Ok(clients)
    } else {
        Err(format!(
            "Number of clients must be {} or less and greater than 0",
            MAX_PLAYERS_IN_TEAM
        ))
    }
}
