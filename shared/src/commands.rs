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
pub enum PlayerCmd {
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

impl TryFrom<&str> for PlayerCmd {
    type Error = String;

    fn try_from(s: &str) -> Result<Self, String> {
        let parts: Vec<&str> = s.splitn(2, ' ').collect();

        //TODO: use lib for handling? for single word command add check that there is only 1 part
        match (parts[0], parts.len()) {
            ("avance" | "move", 1) => Ok(PlayerCmd::Move),
            ("droite" | "right", 1) => Ok(PlayerCmd::Right),
            ("gauche" | "left", 1) => Ok(PlayerCmd::Left),
            ("voir" | "see", 1) => Ok(PlayerCmd::See),
            ("inventaire" | "inv" | "inventory", 1) => Ok(PlayerCmd::Inventory),
            ("prend" | "take", 2) => Ok(PlayerCmd::Take {
                resource_name: parts[1].to_string(),
            }),
            ("pose" | "put", 2) => Ok(PlayerCmd::Put {
                resource_name: parts[1].to_string(),
            }),
            ("expulse" | "expel" | "exp", 1) => Ok(PlayerCmd::Expel),
            ("broadcast" | "bc", 2) => Ok(PlayerCmd::Broadcast {
                text: parts[1].to_string(),
            }),
            ("incantation" | "inc", 1) => Ok(PlayerCmd::Incantation),
            ("fork", 1) => Ok(PlayerCmd::Fork),
            ("connect_nbr" | "cn", 1) => Ok(PlayerCmd::ConnectNbr),
            _ => Err(format!("Unknown command: \"{s}\"")),
        }
    }
}

impl PlayerCmd {
    pub const EGG_FETCH_TIME_DELAY: u64 = 600;
    pub const INCANTATION_DURATION: u64 = 300;

    pub fn delay(&self) -> u64 {
        match self {
            PlayerCmd::Move => 7,
            PlayerCmd::Right => 7,
            PlayerCmd::Left => 7,
            PlayerCmd::See => 7,
            PlayerCmd::Inventory => 1,
            PlayerCmd::Take { .. } => 7,
            PlayerCmd::Put { .. } => 7,
            PlayerCmd::Expel => 7,
            PlayerCmd::Broadcast { .. } => 7,
            PlayerCmd::Incantation => 0,
            PlayerCmd::Fork => 42,
            PlayerCmd::ConnectNbr => 0,
        }
    }
}
