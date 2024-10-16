use clap::Parser;

// TODO: more default values
// TODO: min value for width and height

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
        help = "Number of clients authorized at the beginning of the game"
    )]
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
