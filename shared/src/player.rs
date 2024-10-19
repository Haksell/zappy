use crate::{Command, MAX_COMMANDS};
use crate::{ServerCommandToClient, ZappyError};
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use tokio::sync::mpsc::Sender;

#[derive(Debug)]
pub enum MessageToPlayer {
    ActionQueueIsFull
}

impl MessageToPlayer {
    pub fn get_text(&self) -> &'static str {
        match self {
            _ => "Action queue limit is reached",
        }
    }
}

// TOOD: pos: Pos
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Player {
    #[serde(skip_serializing, skip_deserializing)]
    pub communication_channel: Option<Sender<ServerCommandToClient>>,
    pub team: String,
    pub id: usize,
    pub next_frame: u64,
    pub commands: VecDeque<Command>,
    pub x: usize,
    pub y: usize,
}

impl Player {
    pub fn new(
        communication_channel: Sender<ServerCommandToClient>,
        id: usize,
        team: String,
        x: usize,
        y: usize,
    ) -> Self {
        Self {
            communication_channel: Some(communication_channel),
            id,
            team,
            next_frame: 0,
            commands: VecDeque::with_capacity(MAX_COMMANDS),
            x,
            y,
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
}
