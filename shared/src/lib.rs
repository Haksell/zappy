pub mod commands;
pub mod map;
pub mod player;
pub mod position;
pub mod resource;
pub mod team;
pub mod utils;

use commands::PlayerCommand;
use position::{Direction, Position};
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};

pub const PROJECT_NAME: &'static str = "zappy";

//TODO: move from lib to server
pub enum ZappyError {
    Technical(TechnicalError),
    Logical(LogicalError),
}

pub enum TechnicalError {
    ConnectionClosedByClient(u16),
    ConnectionCorrupted(u16, String),
    AlreadyConnected(u16),
    IsNotConnectedToServer(u16),
    FailedToWriteToSocket(u16, String),
    FailedToReadFromSocket(u16, String),
    MessageCantBeMappedToFromUtf8(u16, String),
    //TODO: delete when the any message size will be handled (or not :P)
    MessageIsTooBig(u16),
}

pub enum LogicalError {
    TeamDoesntExist(String),
    NoPlaceAvailable(u16, String),
    WrongUsernameOrPassword,
}

impl Display for LogicalError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let msg = match self {
            LogicalError::TeamDoesntExist(team) => format!("Team does not exist: {}", team),
            LogicalError::NoPlaceAvailable(_, team_name) => {
                format!("No place available on team {team_name}")
            }
            LogicalError::WrongUsernameOrPassword => "Wrong username or password".to_string(),
        };
        write!(f, "{}", msg)
    }
}

impl Display for TechnicalError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let msg = match self {
            TechnicalError::ConnectionClosedByClient(id) => format!("{id}: client disconnected"),
            TechnicalError::ConnectionCorrupted(id, str) => {
                format!("{id}: connection corrupted: {str}")
            }
            TechnicalError::AlreadyConnected(id) => format!("{id}: already connected"),
            TechnicalError::IsNotConnectedToServer(id) => format!("{id}: is not connected"),
            TechnicalError::FailedToWriteToSocket(id, msg) => format!("{id}: {msg}"),
            TechnicalError::FailedToReadFromSocket(id, msg) => format!("{id}: {msg}"),
            TechnicalError::MessageCantBeMappedToFromUtf8(id, msg) => format!("{id}: {msg}"),
            TechnicalError::MessageIsTooBig(id) => format!("{id}: message is too large"),
        };
        write!(f, "{}", msg)
    }
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
    Movement(Direction),
}

impl Display for ServerResponse {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ServerResponse::Ok => write!(f, "Ok"),
            ServerResponse::Ko => write!(f, "Ko"),
            ServerResponse::Cases(_) => todo!(),
            ServerResponse::Inventory(items) => write!(f, "{{{}}}", items.join(", ")),
            ServerResponse::ElevationInProgress => write!(f, "Elevation InProgress"),
            ServerResponse::Value(value) => write!(f, "{}", value),
            ServerResponse::Mort => write!(f, "Mort"),
            ServerResponse::ActionQueueIsFull => {
                write!(f, "The action queue is full, please try later.")
            }
            ServerResponse::Movement(from) => write!(f, "deplacement {from}"),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Egg {
    pub team_name: String,
    pub position: Position,
}

pub const HANDSHAKE_MSG: &'static str = "BIENVENUE\n";
pub const GFX_PORT: u16 = 4343; // TODO configurable port
pub const ADMIN_PORT: u16 = 4444; // TODO configurable port

pub const MAX_COMMANDS: usize = 10;
pub const MAX_FIELD_SIZE: usize = 50;
pub const MAX_PLAYER_LVL: u8 = 8;
pub const MAX_PLAYERS_IN_TEAM: u16 = 1024;
pub const MAX_TEAMS: u16 = 14; // TODO: sync with ZappyColor

pub const LIFE_TICKS: u64 = 126;
pub const LIVES_START: u64 = 10;
