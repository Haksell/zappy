use serde::{Deserialize, Deserializer, Serialize, Serializer};

#[derive(Debug, Clone, PartialEq)]
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

impl Serialize for Command {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            Command::Avance => serializer.serialize_str("Avance"),
            Command::Droite => serializer.serialize_str("Droite"),
            Command::Gauche => serializer.serialize_str("Gauche"),
            Command::Voir => serializer.serialize_str("Voir"),
            Command::Inventaire => serializer.serialize_str("Inventaire"),
            Command::Prend { resource_name } => {
                serializer.serialize_str(&format!("Prend {}", resource_name))
            }
            Command::Pose { resource_name } => {
                serializer.serialize_str(&format!("Pose {}", resource_name))
            }
            Command::Expulse => serializer.serialize_str("Expulse"),
            Command::Broadcast { text } => serializer.serialize_str(&format!("Broadcast {}", text)),
            Command::Incantation => serializer.serialize_str("Incantation"),
            Command::Fork => serializer.serialize_str("Fork"),
            Command::ConnectNbr => serializer.serialize_str("ConnectNbr"),
        }
    }
}

impl<'de> Deserialize<'de> for Command {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        log::info!("command: {}", s);
        let parts: Vec<&str> = s.splitn(2, ' ').collect();

        match parts[0] {
            "Avance" => Ok(Command::Avance),
            "Droite" => Ok(Command::Droite),
            "Gauche" => Ok(Command::Gauche),
            "Voir" => Ok(Command::Voir),
            "Inventaire" => Ok(Command::Inventaire),
            "Prend" => {
                if parts.len() == 2 {
                    Ok(Command::Prend {
                        resource_name: parts[1].to_string(),
                    })
                } else {
                    Err(serde::de::Error::custom("Expected resource name for Prend"))
                }
            }
            "Pose" => {
                if parts.len() == 2 {
                    Ok(Command::Pose {
                        resource_name: parts[1].to_string(),
                    })
                } else {
                    Err(serde::de::Error::custom("Expected resource name for Pose"))
                }
            }
            "Expulse" => Ok(Command::Expulse),
            "Broadcast" => {
                if parts.len() == 2 {
                    Ok(Command::Broadcast {
                        text: parts[1].to_string(),
                    })
                } else {
                    Err(serde::de::Error::custom("Expected text for Broadcast"))
                }
            }
            "Incantation" => Ok(Command::Incantation),
            "Fork" => Ok(Command::Fork),
            "ConnectNbr" => Ok(Command::ConnectNbr),
            _ => Err(serde::de::Error::custom("Unknown command")),
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
