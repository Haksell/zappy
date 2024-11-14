use crate::{resource::Resource, Command, ServerCommandToClient, ZappyError, MAX_COMMANDS};
use derive_getters::Getters;
use rand::seq::SliceRandom;
use rand::thread_rng;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use tokio::sync::mpsc::Sender;

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub enum Direction {
    North,
    East,
    South,
    West,
}

pub enum Side {
    Left,
    Right,
}

impl Direction {
    pub fn random() -> Self {
        static DIRECTIONS: [Direction; 4] = [
            Direction::North,
            Direction::East,
            Direction::South,
            Direction::West,
        ];

        let mut rng = thread_rng();
        *DIRECTIONS.choose(&mut rng).unwrap()
    }

    pub fn turn(&self, side: Side) -> Self {
        match side {
            Side::Left => match self {
                Direction::North => Direction::West,
                Direction::West => Direction::South,
                Direction::South => Direction::East,
                Direction::East => Direction::North,
            },
            Side::Right => match self {
                Direction::North => Direction::East,
                Direction::East => Direction::South,
                Direction::South => Direction::West,
                Direction::West => Direction::North,
            },
        }
    }

    pub fn dx_dy(&self) -> (isize, isize) {
        match self {
            Direction::North => (0, -1),
            Direction::East => (1, 0),
            Direction::South => (0, 1),
            Direction::West => (-1, 0),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub struct Position {
    pub x: usize,
    pub y: usize,
    pub direction: Direction,
}

// TOOD: pos: Pos
#[derive(Getters, Serialize, Deserialize, Debug, Clone)]
pub struct Player {
    //TODO: communication channel is still unused, can be used in case of die, admin disconnect
    #[serde(skip_serializing, skip_deserializing)]
    communication_channel: Option<Sender<ServerCommandToClient>>,
    team: String,
    id: u16,
    next_frame: u64,
    commands: VecDeque<Command>,
    pub(crate) position: Position,
    inventory: [usize; Resource::SIZE],
}

impl Player {
    pub fn new(
        communication_channel: Sender<ServerCommandToClient>,
        id: u16,
        team: String,
        position: Position,
    ) -> Self {
        Self {
            communication_channel: Some(communication_channel),
            id,
            team,
            next_frame: 0,
            commands: VecDeque::with_capacity(MAX_COMMANDS),
            position,
            inventory: [0; Resource::SIZE],
        }
    }

    pub async fn disconnect(&self) -> Result<(), ZappyError> {
        self.communication_channel
            .as_ref()
            .unwrap()
            .send(ServerCommandToClient::Shutdown)
            .await
            .map_err(|e| {
                log::error!("[err while disconnect] {}", e);
                ZappyError::ConnectionCorrupted
            })
    }

    pub fn pop_command_from_queue(&mut self) -> Option<Command> {
        // not an Option?
        self.commands.pop_front()
    }

    pub fn push_command_to_queue(&mut self, command: Command) {
        self.commands.push_back(command);
    }

    pub fn set_next_frame(&mut self, value: u64) {
        self.next_frame = value;
    }

    pub fn add_to_inventory(&mut self, resource: Resource) {
        self.inventory[resource as usize] += 1;
    }

    pub fn remove_from_inventory(&mut self, resource: Resource) -> bool {
        if self.inventory[resource as usize] >= 1 {
            self.inventory[resource as usize] -= 1;
            true
        } else {
            false
        }
    }
}
