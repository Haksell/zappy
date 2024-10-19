use crate::{Command, MAX_COMMANDS};
use crate::{ServerCommandToClient, ZappyError};
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use tokio::sync::mpsc::Sender;

// TOOD: pos: Pos
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Player {
    //TODO: communication channel is still unused, can be used in case of die, admin disconnect
    #[serde(skip_serializing, skip_deserializing)]
    pub communication_channel: Option<Sender<ServerCommandToClient>>,
    pub team: String,
    id: u16,
    pub next_frame: u64,
    pub commands: VecDeque<Command>,
    pub x: usize,
    pub y: usize
}

impl Player {
    pub fn new(
        communication_channel: Sender<ServerCommandToClient>,
        id: u16,
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
    
    pub fn id(&self) -> u16 {
        self.id
    }
}
