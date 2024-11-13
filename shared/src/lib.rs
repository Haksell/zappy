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

impl ServerResponse {
    pub fn get_text(&self) -> &'static str {
        match self {
            ServerResponse::Ok => "Ok",
            ServerResponse::Ko => "Ko",
            ServerResponse::Cases(_) => todo!(),
            ServerResponse::Inventory(_) => todo!(),
            ServerResponse::ElevationInProgress => "Elevation InProgress",
            ServerResponse::Value(_) => todo!(),
            ServerResponse::Mort => "Mort",
            ServerResponse::ActionQueueIsFull => "The action queue is full, please try later.",
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
    pub map: Vec<Vec<Cell>>,
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
        Self { map, width, height }
    }

    pub fn random_position(&self) -> Position {
        let mut thread_rng = rand::thread_rng();
        Position {
            x: thread_rng.gen_range(0..self.width),
            y: thread_rng.gen_range(0..self.height),
            direction: Direction::random(),
        }
    }

    fn handle_avance(&mut self, player: &mut Player) {
        let current_x = player.position.x;
        let current_y = player.position.y;

        let (dx, dy) = player.position.direction.dx_dy();
        player.position.x = ((current_x + self.width) as isize + dx) as usize % self.width;
        player.position.y = ((current_y + self.height) as isize + dy) as usize % self.height;

        self.map[current_y][current_x].players.remove(player.id());
        self.map[player.position.y][player.position.x]
            .players
            .insert(*player.id());
    }

    pub fn apply_cmd(&mut self, player: &mut Player, command: &Command) -> Option<ServerResponse> {
        log::debug!("Executing command: {:?} for {:?}", command, player);
        match command {
            Command::Gauche => {
                player.position.direction = player.position.direction.turn(Side::Left);
                Some(ServerResponse::Ok)
            }
            Command::Droite => {
                player.position.direction = player.position.direction.turn(Side::Right);
                Some(ServerResponse::Ok)
            }
            Command::Avance => {
                self.handle_avance(player);
                Some(ServerResponse::Ok)
            }
            Command::Prend { resource_name } => {
                if let Ok(resource) = Resource::try_from(resource_name.as_str()) {
                    let cell = &mut self.map[player.position.y][player.position.x];
                    if cell.resources[resource as usize] >= 1 {
                        cell.resources[resource as usize] -= 1;
                        player.add_to_inventory(resource);
                        return Some(ServerResponse::Ok);
                    }
                }
                Some(ServerResponse::Ko)
            }
            Command::Pose { resource_name } => {
                if let Ok(resource) = Resource::try_from(resource_name.as_str()) {
                    let cell = &mut self.map[player.position.y][player.position.x];
                    if player.remove_from_inventory(resource) {
                        cell.resources[resource as usize] += 1;
                        return Some(ServerResponse::Ok);
                    }
                }
                Some(ServerResponse::Ko)
            }
            Command::Voir => todo!(),
            Command::Inventaire => todo!(),
            Command::Expulse => todo!(),
            Command::Broadcast { text } => todo!(),
            Command::Incantation => todo!(),
            Command::Fork => todo!(),
            Command::ConnectNbr => todo!(),
        }
    }

    pub fn add_player(&mut self, id: u16, position: &Position) {
        log::debug!("Adding {} to the game field.", id);
        self.map[position.y][position.x].players.insert(id);
    }

    pub fn remove_player(&mut self, id: &u16, position: &Position) {
        log::debug!("Removing {} from the game field.", id);
        self.map[position.y][position.x].players.remove(id);
    }
}

pub const GFX_PORT: u16 = 4343;
pub const MAX_COMMANDS: usize = 10;
pub const MAX_FIELD_SIZE: usize = 50;
pub const HANDSHAKE_MSG: &'static str = "BIENVENUE\n";
