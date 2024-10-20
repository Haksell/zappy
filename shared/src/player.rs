use crate::{Command, MAX_COMMANDS};
use crate::{ServerCommandToClient, ZappyError};
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use derive_getters::Getters;
use rand::seq::SliceRandom;
use rand::thread_rng;
use tokio::sync::mpsc::Sender;

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub enum Direction {
    North,
    South,
    East,
    West
}

impl Direction {
    const DIRECTIONS: [Direction; 4] = [Direction::North, Direction::South, Direction::East, Direction::West];
    pub fn random() -> Self {
        let mut rng = thread_rng();
        *Direction::DIRECTIONS.choose(&mut rng).unwrap()
    }
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
    x: usize,
    y: usize,
    direction: Direction
}

impl Player {
    pub fn new(
        communication_channel: Sender<ServerCommandToClient>,
        id: u16,
        team: String,
        x: usize,
        y: usize,
        direction: Direction
    ) -> Self {
        Self {
            communication_channel: Some(communication_channel),
            id,
            team,
            next_frame: 0,
            commands: VecDeque::with_capacity(MAX_COMMANDS),
            x,
            y,
            direction
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
