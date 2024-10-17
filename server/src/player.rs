use crate::{ServerCommandToClient, ZappyError};
use shared::{Command, MAX_COMMANDS};
use std::collections::VecDeque;
use tokio::sync::mpsc::Sender;

/*
STACK
abc
abcde
ab
a

QUEUE
abc
bc
bcde
*/

pub struct Player {
    communication_channel: Sender<ServerCommandToClient>,
    pub(crate) team: String,
    pub(crate) id: usize,
    pub(crate) next_frame: u64,
    pub(crate) commands: VecDeque<Command>,
}

impl Player {
    pub fn new(
        communication_channel: Sender<ServerCommandToClient>,
        id: usize,
        team: String,
    ) -> Self {
        Self {
            communication_channel,
            id,
            team,
            next_frame: 0,
            commands: VecDeque::with_capacity(MAX_COMMANDS),
        }
    }

    pub async fn disconnect(&self) -> Result<(), ZappyError> {
        self.communication_channel
            .send(ServerCommandToClient::Shutdown)
            .await
            .map_err(|e| {
                log::error!("[err while disconnect] {}", e);
                ZappyError::ConnectionCorrupted
            })
    }

    pub fn execute(&self, command: &Command) {
        log::debug!("{command:?}");
    }
}
