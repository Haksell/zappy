pub mod player;

use player::Player;
use rand::Rng as _;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use crate::player::MessageToPlayer;

#[derive(Debug)]
pub enum ZappyError {
    ConnectionClosedByClient,
    MaxPlayersReached,
    ConnectionCorrupted,
    AlreadyConnected,
    TryToDisconnectNotConnected,
    TeamDoesntExist,
    IsNotConnectedToServer,
    TechnicalError(String),
    Waring(MessageToPlayer)
}

pub enum ServerCommandToClient {
    Shutdown,
    SendMessage(String),
}

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ServerResponse {
    Ok,
    Ko,
    Cases(Vec<String>),
    Inventory(Vec<String>),
    ElevationInProgress,
    Value(String),
    Mort,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum Command {
    Avance,
    Droite,
    Gauche,
    Voir,
    Inventaire,
    Prend { object_name: String },
    Pose { object_name: String },
    Expulse,
    Broadcast { text: String },
    Incantation,
    Fork,
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

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Hash, Clone)]
pub enum Resource {
    Linemate,
    Deraumere,
    Sibur,
    Mendiane,
    Phiras,
    Thystame,
    Nourriture,
}

impl Resource {
    pub fn alias(&self) -> char {
        match self {
            Resource::Linemate => 'L',
            Resource::Deraumere => 'D',
            Resource::Sibur => 'S',
            Resource::Mendiane => 'M',
            Resource::Phiras => 'P',
            Resource::Thystame => 'T',
            Resource::Nourriture => 'N',
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Egg {
    pub team_name: String,
    pub start_frame: u64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Cell {
    pub players: Vec<Arc<Player>>,
    pub resources: HashMap<Resource, usize>,
    pub eggs: Vec<Egg>,
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

    pub fn random_position(&self) -> (usize, usize) {
        let mut thread_rng = rand::thread_rng();
        (
            thread_rng.gen_range(0..self.width),
            thread_rng.gen_range(0..self.height),
        )
    }

    pub fn add_player(&mut self, player: Arc<Player>) {
        println!("add_player: {player:?}");
        self.map[player.y][player.x].players.push(player); // TODO: Arc<Mutex> or some shit
    }
}

pub const GFX_PORT: u16 = 4343;
pub const MAX_COMMANDS: usize = 10;
pub const MAX_FIELD_SIZE: usize = 50;
pub const HANDSHAKE_MSG: &'static str = "BIENVENUE\n";
