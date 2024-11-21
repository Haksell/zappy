use serde::{Deserialize, Serialize};

pub enum AdminCommand {
    ShowOff,
}

impl AdminCommand {
    pub fn show_off(&self)  {
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
    Avance,
    Droite,
    Gauche,
    Voir,
    Inventaire,
    Prend { resource_name: String },
    Pose { resource_name: String },
    Expulse,
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
        match parts[0] {
            "avance" => Ok(PlayerCommand::Avance),
            "droite" => Ok(PlayerCommand::Droite),
            "gauche" => Ok(PlayerCommand::Gauche),
            "voir" => Ok(PlayerCommand::Voir),
            "inventaire" => Ok(PlayerCommand::Inventaire),
            "prend" => {
                if parts.len() == 2 {
                    Ok(PlayerCommand::Prend {
                        resource_name: parts[1].to_string(),
                    })
                } else {
                    Err("Expected resource name for Prend".to_string())
                }
            }
            "pose" => {
                if parts.len() == 2 {
                    Ok(PlayerCommand::Pose {
                        resource_name: parts[1].to_string(),
                    })
                } else {
                    Err("Expected resource name for Pose".to_string())
                }
            }
            "expulse" => Ok(PlayerCommand::Expulse),
            "broadcast" => {
                if parts.len() == 2 {
                    Ok(PlayerCommand::Broadcast {
                        text: parts[1].to_string(),
                    })
                } else {
                    Err("Expected text for Broadcast".to_string())
                }
            }
            "incantation" => Ok(PlayerCommand::Incantation),
            "fork" => Ok(PlayerCommand::Fork),
            "connect_nbr" => Ok(PlayerCommand::ConnectNbr),
            _ => Err(format!("Unknown command: \"{s}\"")),
        }
    }
}

impl PlayerCommand {
    pub fn delay(&self) -> u64 {
        match self {
            PlayerCommand::Avance => 7,
            PlayerCommand::Droite => 7,
            PlayerCommand::Gauche => 7,
            PlayerCommand::Voir => 7,
            PlayerCommand::Inventaire => 1,
            PlayerCommand::Prend { .. } => 7,
            PlayerCommand::Pose { .. } => 7,
            PlayerCommand::Expulse => 7,
            PlayerCommand::Broadcast { .. } => 7,
            PlayerCommand::Incantation => 300,
            PlayerCommand::Fork => 42,
            PlayerCommand::ConnectNbr => 0,
        }
    }
}
