pub mod command;
pub mod map;
pub mod player;
pub mod resource;

use command::Command;
use serde::{Deserialize, Serialize};
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

pub const GFX_PORT: u16 = 4343; // TODO configurable port
pub const MAX_COMMANDS: usize = 10;
pub const MAX_FIELD_SIZE: usize = 50;
pub const HANDSHAKE_MSG: &'static str = "BIENVENUE\n";
