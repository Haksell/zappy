use clap::Parser;
use derive_builder::Builder;
use shared::{MAX_PLAYERS_IN_TEAM, MAX_TEAMS};

// TODO: more default values

#[derive(Parser, Debug, Builder, Default)]
#[builder(setter(into))]
#[command(version, about, long_about = None)]
pub(crate) struct ServerArgs {
    #[arg(short, long, help = "Port number", default_value_t = 8080)]
    pub(crate) port: u16,

    #[arg(short('x'), long, value_parser = validate_dimension, help = "World width")]
    pub(crate) width: usize,

    #[arg(short('y'), long, value_parser = validate_dimension, help = "World height")]
    pub(crate) height: usize,

    #[arg(
        short,
        long,
        default_value_t = 1,
        value_parser = validate_clients,
        help = "Number of clients per team authorized at the beginning of the game"
    )]
    pub(crate) clients: u16,

    #[arg(
        short,
        long,
        help = "Time Unit Divider (the greater t is, the faster the game will go)",
        default_value_t = 100
    )]
    pub(crate) tud: u16,

    #[arg(
        short,
        long,
        help = "List of team names",
        required = true,
        num_args = 1..=MAX_TEAMS
    )]
    pub(crate) names: Vec<String>,
}

fn validate_dimension(s: &str) -> Result<usize, String> {
    let dimension: usize = s.parse().map_err(|_| "Not a valid number")?;
    if (2..=100).contains(&dimension) {
        Ok(dimension)
    } else {
        Err(format!("Grid dimensions must be between 2 and 100"))
    }
}

fn validate_clients(s: &str) -> Result<u16, String> {
    let clients: u16 = s.parse().map_err(|_| "Not a valid number")?;
    if (1..=MAX_PLAYERS_IN_TEAM).contains(&clients) {
        Ok(clients)
    } else {
        Err(format!(
            "Number of clients must be between 1 and {MAX_PLAYERS_IN_TEAM}",
        ))
    }
}
