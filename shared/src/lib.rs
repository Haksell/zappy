use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Represents the different types of responses the server can send to the client.
/// Review comments
#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ServerResponse {
    /// Indicates a successful operation.
    Ok,
    /// Indicates a failed operation.
    Ko,
    /// Represents the response from the `see` command.
    /// Contains a list of visible cases.
    Cases(Vec<String>),
    /// Represents the response from the `inventory` command.
    /// Contains a list of inventory items with their quantities.
    Inventory(Vec<String>),
    /// Indicates that an incantation (elevation) is in progress.
    ElevationInProgress,
    /// Represents a generic value response.
    Value(String),
    ///Death of a player
    Mort,
}

/// Represents the different commands that can be sent to the server.
#[derive(Debug, Serialize, Deserialize)]
pub enum Command {
    /// Advance one square.
    /// Command: avance
    Avance,

    /// Turn right 90 degrees.
    /// Command: degrees_droite
    Droite,

    /// Turn left 90 degrees.
    /// Command: degrees_gauche
    Gauche,

    /// See the surrounding squares.
    /// Command: voir
    Voir,

    /// View inventory.
    /// Command: inventaire
    Inventaire,

    /// Take an object.
    /// Command: prend <object>
    Prend {
        /// The object to take.
        object_name: String,
    },

    /// Put down an object.
    /// Command: pose <object>
    Pose {
        /// The object to put down.
        object_name: String,
    },

    /// Kick the players from the square.
    /// Command: expulse
    Expulse,

    /// Broadcast a message.
    /// Command: broadcast <text>
    Broadcast {
        /// The message to broadcast.
        text: String,
    },

    /// Begin the incantation (elevation).
    /// Command: incantation
    Incantation,

    /// Fork a player.
    /// Command: fork
    Fork,

    /// Know the number of unused connections by the team.
    /// Command: connect_nbr
    ConnectNbr,
}

impl Command {
    pub fn delay(&self) -> u64 {
        match self {
            Command::Avance => 7,
            Command::Droite => 7,
            Command::Gauche => 7,
            Command::Voir => 7,
            Command::Inventaire => 1,
            Command::Prend { .. } => 7,
            Command::Pose { .. } => 7,
            Command::Expulse => 7,
            Command::Broadcast { .. } => 7,
            Command::Incantation => 300,
            Command::Fork => 42,
            Command::ConnectNbr => 0,
        }
    }
}

type Unknown = String;

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Hash, Clone)]
enum Resource {
    Linemate,
    Deraumere,
    Sibur,
    Mendiane,
    Phiras,
    Thystame,
    Nourriture,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct Egg {
    team_name: String,
    start_frame: u64,
}
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Cell {
    players: Vec<Unknown>,
    resources: HashMap<Resource, usize>,
    eggs: Vec<Egg>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Map {
    pub map: Vec<Vec<Cell>>,
    pub width: usize,
    pub height: usize,
}

impl Cell {
    pub fn new() -> Self {
        Self {
            players: Vec::new(),
            resources: HashMap::new(),
            eggs: Vec::new(),
        }
    }
}
impl Map {
    pub fn new(width: usize, height: usize) -> Self {
        let map = vec![vec![Cell::new(); width]; height];
        Self { map, width, height }
    }
}
pub const GFX_PORT: u16 = 4343;
pub const MAX_COMMANDS: usize = 10;
