use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Command {
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

impl TryFrom<&str> for Command {
    type Error = String;

    fn try_from(s: &str) -> Result<Self, String> {
        log::info!("command: {}", s);
        let parts: Vec<&str> = s.splitn(2, ' ').collect();

        match parts[0] {
            "avance" => Ok(Command::Avance),
            "droite" => Ok(Command::Droite),
            "gauche" => Ok(Command::Gauche),
            "voir" => Ok(Command::Voir),
            "inventaire" => Ok(Command::Inventaire),
            "prend" => {
                if parts.len() == 2 {
                    Ok(Command::Prend {
                        resource_name: parts[1].to_string(),
                    })
                } else {
                    Err("Expected resource name for Prend".to_string())
                }
            }
            "pose" => {
                if parts.len() == 2 {
                    Ok(Command::Pose {
                        resource_name: parts[1].to_string(),
                    })
                } else {
                    Err("Expected resource name for Pose".to_string())
                }
            }
            "expulse" => Ok(Command::Expulse),
            "broadcast" => {
                if parts.len() == 2 {
                    Ok(Command::Broadcast {
                        text: parts[1].to_string(),
                    })
                } else {
                    Err("Expected text for Broadcast".to_string())
                }
            }
            "incantation" => Ok(Command::Incantation),
            "fork" => Ok(Command::Fork),
            "connect_nbr" => Ok(Command::ConnectNbr),
            _ => Err("Unknown command".to_string()),
        }
    }
}

impl Command {
    pub fn delay(&self) -> u64 {
        match self {
            Command::Avance => 7,
            Command::Droite => 7,
            Command::Gauche => 7,
            Command::Voir => 7,
            Command::Inventaire => 1,
            Command::Prend { .. } => 7,
            Command::Pose { .. } => 7,
            Command::Expulse => 7,
            Command::Broadcast { .. } => 7,
            Command::Incantation => 300,
            Command::Fork => 42,
            Command::ConnectNbr => 0,
        }
    }
}
