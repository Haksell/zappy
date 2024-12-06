pub mod color;
pub mod commands;
pub mod map;
pub mod player;
pub mod position;
pub mod resource;
pub mod team;
pub mod utils;

use color::ZappyColor;
use map::Map;
use player::Player;
use position::{Direction, Position};
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeMap,
    fmt::{Display, Formatter},
};

pub const PROJECT_NAME: &'static str = "zappy";

#[derive(Debug, Default)]
pub struct ServerData {
    pub map: Map,
    pub players: BTreeMap<u16, Player>,
    pub teams: BTreeMap<String, (ZappyColor, usize)>,
}

impl ServerData {
    pub fn new(map: Map, players: BTreeMap<u16, Player>, teams: BTreeMap<String, usize>) -> Self {
        let teams = teams
            .iter()
            .enumerate()
            .map(|(i, (name, &members_count))| (name.clone(), (ZappyColor::idx(i), members_count)))
            .collect::<BTreeMap<String, (ZappyColor, usize)>>();
        Self {
            map,
            players,
            teams,
        }
    }
}

//TODO: move from lib to server
#[derive(Debug, PartialEq)]
pub enum ZappyError {
    Network(NetworkError),
    Game(GameError),
    Player(PlayerError),
}

#[derive(Debug, PartialEq)]
pub enum NetworkError {
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

#[derive(Debug, PartialEq)]
pub enum GameError {
    IncreasingLevelButIsAlreadyMax(u16),
    IncreasingLevelWithNoIncantations(u16),
}

#[derive(Debug, PartialEq)]
pub enum PlayerError {
    TeamDoesntExist(String),
    NoPlaceAvailable(u16, String),
    WrongUsernameOrPassword,
}

impl Display for GameError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let msg = match self {
            GameError::IncreasingLevelWithNoIncantations(id) => {
                format!("{id}: trying to stop incantation, but no incantation is happening")
            }
            GameError::IncreasingLevelButIsAlreadyMax(id) => {
                format!("{id}: trying to stop incantation, but the max level is already reached")
            }
        };
        write!(f, "{}", msg)
    }
}

impl Display for NetworkError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let msg = match self {
            NetworkError::ConnectionClosedByClient(id) => format!("{id}: client disconnected"),
            NetworkError::ConnectionCorrupted(id, str) => {
                format!("{id}: connection corrupted: {str}")
            }
            NetworkError::AlreadyConnected(id) => format!("{id}: already connected"),
            NetworkError::IsNotConnectedToServer(id) => format!("{id}: is not connected"),
            NetworkError::FailedToWriteToSocket(id, msg) => format!("{id}: {msg}"),
            NetworkError::FailedToReadFromSocket(id, msg) => format!("{id}: {msg}"),
            NetworkError::MessageCantBeMappedToFromUtf8(id, msg) => format!("{id}: {msg}"),
            NetworkError::MessageIsTooBig(id) => format!("{id}: message is too large"),
        };
        write!(f, "{}", msg)
    }
}

impl Display for PlayerError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let msg = match self {
            PlayerError::TeamDoesntExist(team) => format!("Team does not exist: {}", team),
            PlayerError::NoPlaceAvailable(_, team_name) => {
                format!("No place available on team {team_name}")
            }
            PlayerError::WrongUsernameOrPassword => "Wrong username or password".to_string(),
        };
        write!(f, "{}", msg)
    }
}

pub enum ServerCommandToClient {
    Shutdown,
    SendMessage(ServerResponse),
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub enum ServerResponse {
    Ok,
    Ko,
    Cases(Vec<String>),
    Inventory(Vec<String>),
    See(Vec<String>),
    IncantationInProgress,
    CurrentLevel(u8),
    Value(String),
    Mort,
    ActionQueueIsFull,
    Movement(Direction),
    Message(u8, String),
}

impl Display for ServerResponse {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ServerResponse::Ok => write!(f, "Ok"),
            ServerResponse::Ko => write!(f, "Ko"),
            ServerResponse::Cases(_) => todo!(),
            ServerResponse::Inventory(items) | ServerResponse::See(items) => {
                write!(f, "{{{}}}", items.join(", "))
            }
            ServerResponse::IncantationInProgress => write!(f, "elevation en cours"),
            ServerResponse::CurrentLevel(level) => write!(f, "niveau actuel : {level}"),
            ServerResponse::Value(value) => write!(f, "{}", value),
            ServerResponse::Mort => write!(f, "Mort"),
            ServerResponse::ActionQueueIsFull => {
                write!(f, "The action queue is full, please try later.")
            }
            ServerResponse::Movement(from) => write!(f, "deplacement {from}"),
            ServerResponse::Message(source, text) => write!(f, "message {source},{text}"),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
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
pub const DECREASED_HP_PER_FRAME: u64 = 1;
pub const MAX_PLAYERS_IN_TEAM: u16 = 1024;
pub const MAX_TEAMS: u16 = 14; // TODO: sync with ZappyColor

pub const LIFE_TICKS: u64 = 444 * 126;
pub const LIVES_START: u64 = 10;
