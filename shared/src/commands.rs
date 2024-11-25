use serde::{Deserialize, Serialize};

pub enum AdminCommand {
    ShowOff,
}

impl AdminCommand {
    pub fn show_off(&self) {
        log::info!("Admin showing off")
    }
}

impl TryFrom<&str> for AdminCommand {
    type Error = String;

    fn try_from(s: &str) -> Result<Self, String> {
        let parts: Vec<&str> = s.splitn(2, ' ').collect();

        match (parts[0], parts.len()) {
            ("show_off", 1) => Ok(AdminCommand::ShowOff),
            _ => Err(format!("Unknown command: \"{s}\"")),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Hash)]
pub enum PlayerCommand {
    Move,
    Right,
    Left,
    See,
    Inventory,
    Take { resource_name: String },
    Put { resource_name: String },
    Expel,
    Broadcast { text: String },
    Incantation,
    Fork,
    ConnectNbr,
}

impl TryFrom<&str> for PlayerCommand {
    type Error = String;

    fn try_from(s: &str) -> Result<Self, String> {
        let parts: Vec<&str> = s.splitn(2, ' ').collect();

        //TODO: use lib for handling? for single word command add check that there is only 1 part
        match (parts[0], parts.len()) {
            ("avance" | "move", 1) => Ok(PlayerCommand::Move),
            ("droite" | "right", 1) => Ok(PlayerCommand::Right),
            ("gauche" | "left", 1) => Ok(PlayerCommand::Left),
            ("voir" | "see", 1) => Ok(PlayerCommand::See),
            ("inventaire" | "inv" | "inventory", 1) => Ok(PlayerCommand::Inventory),
            ("prend" | "take", 2) => Ok(PlayerCommand::Take {
                resource_name: parts[1].to_string(),
            }),
            ("pose" | "put", 2) => Ok(PlayerCommand::Put {
                resource_name: parts[1].to_string(),
            }),
            ("expulse" | "expel" | "exp", 1) => Ok(PlayerCommand::Expel),
            ("broadcast", 2) => Ok(PlayerCommand::Broadcast {
                text: parts[1].to_string(),
            }),
            ("incantation", 1) => Ok(PlayerCommand::Incantation),
            ("fork", 1) => Ok(PlayerCommand::Fork),
            ("connect_nbr" | "cn", 1) => Ok(PlayerCommand::ConnectNbr),
            _ => Err(format!("Unknown command: \"{s}\"")),
        }
    }
}

impl PlayerCommand {
    pub const EGG_FETCH_TIME_DELAY: u64 = 600;
    pub const INCANTATION_DURATION: u64 = 300;

    pub fn delay(&self) -> u64 {
        match self {
            PlayerCommand::Move => 7,
            PlayerCommand::Right => 7,
            PlayerCommand::Left => 7,
            PlayerCommand::See => 7,
            PlayerCommand::Inventory => 1,
            PlayerCommand::Take { .. } => 7,
            PlayerCommand::Put { .. } => 7,
            PlayerCommand::Expel => 7,
            PlayerCommand::Broadcast { .. } => 7,
            PlayerCommand::Incantation => 0,
            PlayerCommand::Fork => 42,
            PlayerCommand::ConnectNbr => 0,
        }
    }
}
