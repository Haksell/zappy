use crate::{ServerCommandToClient, ZappyError};
use tokio::sync::mpsc::Sender;

pub struct Player {
    communication_channel: Sender<ServerCommandToClient>,
    pub(crate) team: String,
    pub(crate) id: usize,
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
}
