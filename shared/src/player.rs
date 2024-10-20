use crate::{Command, MAX_COMMANDS};
use crate::{ServerCommandToClient, ZappyError};
use derive_getters::Getters;
use rand::seq::SliceRandom;
use rand::thread_rng;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use tokio::sync::mpsc::Sender;

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub enum Direction {
    North,
    South,
    East,
    West,
}

pub enum Side {
    Left,
    Right,
}

impl Direction {
    const DIRECTIONS: [Direction; 4] = [
        Direction::North,
        Direction::South,
        Direction::East,
        Direction::West,
    ];
    pub fn random() -> Self {
        let mut rng = thread_rng();
        *Direction::DIRECTIONS.choose(&mut rng).unwrap()
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
        self.commands.pop_front()
    }

    pub fn push_command_to_queue(&mut self, command: Command) {
        self.commands.push_back(command);
    }

    pub fn set_next_frame(&mut self, value: u64) {
        self.next_frame = value;
    }
}
