pub mod command;
pub mod player;
pub mod resource;

use crate::player::{Direction, Position, Side};
use command::Command;
use player::Player;
use rand::Rng as _;
use resource::Resource;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fmt::{Display, Formatter};

#[derive(Debug)]
pub enum ZappyError {
    ConnectionClosedByClient,
    MaxPlayersReached,
    ConnectionCorrupted,
    AlreadyConnected,
    TeamDoesntExist(String),
    IsNotConnectedToServer,
    TechnicalError(String),
}

pub enum ServerCommandToClient {
    Shutdown,
    SendMessage(ServerResponse),
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
    ActionQueueIsFull,
}

impl Display for ServerResponse {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ServerResponse::Ok => write!(f, "Ok"),
            ServerResponse::Ko => write!(f, "Ko"),
            ServerResponse::Cases(_) => todo!(),
            ServerResponse::Inventory(items) => write!(f, "{{{}}}", items.join(", ")),
            ServerResponse::ElevationInProgress => write!(f, "Elevation InProgress"),
            ServerResponse::Value(_) => todo!(),
            ServerResponse::Mort => write!(f, "Mort"),
            ServerResponse::ActionQueueIsFull => {
                write!(f, "The action queue is full, please try later.")
            }
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
    pub players: HashSet<u16>,
    pub resources: [usize; Resource::SIZE],
    pub eggs: Vec<Egg>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Map {
    pub field: Vec<Vec<Cell>>,
    pub width: usize,
    pub height: usize,
}

impl Cell {
    pub fn new() -> Self {
        Self {
            players: HashSet::new(),
            resources: [0; Resource::SIZE],
            eggs: Vec::new(),
        }
    }

    pub fn add_resource(&mut self, resource: Resource) {
        self.resources[resource as usize] += 1;
    }
}

impl Map {
    // TODO: better procedural generation
    pub fn new(width: usize, height: usize) -> Self {
        let mut map = vec![vec![Cell::new(); width]; height];
        for y in 0..height {
            for x in 0..width {
                map[y][x].add_resource(Resource::random());
            }
        }
        Self { field: map, width, height }
    }

    pub fn random_position(&self) -> Position {
        let mut thread_rng = rand::thread_rng();
        Position {
            x: thread_rng.gen_range(0..self.width),
            y: thread_rng.gen_range(0..self.height),
            direction: Direction::random(),
        }
    }

    

    pub fn add_player(&mut self, id: u16, position: &Position) {
        log::debug!("Adding {} to the game field.", id);
        self.field[position.y][position.x].players.insert(id);
    }

    pub fn remove_player(&mut self, id: &u16, position: &Position) {
        log::debug!("Removing {} from the game field.", id);
        self.field[position.y][position.x].players.remove(id);
    }
}

pub const GFX_PORT: u16 = 4343; // TODO configurable port
pub const MAX_COMMANDS: usize = 10;
pub const MAX_FIELD_SIZE: usize = 50;
pub const HANDSHAKE_MSG: &'static str = "BIENVENUE\n";
