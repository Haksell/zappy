use rand::{seq::SliceRandom as _, thread_rng};
use serde::{Deserialize, Serialize};
use std::fmt::Display;

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Hash, Clone, Copy)]
#[repr(u8)]
pub enum Resource {
    Deraumere,
    Linemate,
    Mendiane,
    Nourriture,
    Phiras,
    Sibur,
    Thystame,
}

impl Resource {
    pub const SIZE: usize = 7; // TODO: dynamic

    pub fn alias(&self) -> char {
        match self {
            Resource::Linemate => 'L',
            Resource::Deraumere => 'D',
            Resource::Sibur => 'S',
            Resource::Mendiane => 'M',
            Resource::Phiras => 'P',
            Resource::Thystame => 'T',
            Resource::Nourriture => 'N',
        }
    }

    pub fn random() -> Self {
        static RESOURCES: [Resource; Resource::SIZE] = [
            Resource::Deraumere,
            Resource::Linemate,
            Resource::Mendiane,
            Resource::Nourriture,
            Resource::Phiras,
            Resource::Sibur,
            Resource::Thystame,
        ];

        let mut rng = thread_rng();
        *RESOURCES.choose(&mut rng).unwrap()
    }
}

impl Display for Resource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str = match self {
            Resource::Deraumere => "Deraumere",
            Resource::Linemate => "Linemate",
            Resource::Mendiane => "Mendiane",
            Resource::Nourriture => "Nourriture",
            Resource::Phiras => "Phiras",
            Resource::Sibur => "Sibur",
            Resource::Thystame => "Thystame",
        }
        .to_string();
        write!(f, "{}", str)
    }
}

impl TryFrom<u8> for Resource {
    type Error = &'static str;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Resource::Deraumere),
            1 => Ok(Resource::Linemate),
            2 => Ok(Resource::Mendiane),
            3 => Ok(Resource::Nourriture),
            4 => Ok(Resource::Phiras),
            5 => Ok(Resource::Sibur),
            6 => Ok(Resource::Thystame),
            _ => unreachable!(),
        }
    }
}

impl TryFrom<&str> for Resource {
    type Error = ();

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "Deraumere" => Ok(Resource::Deraumere),
            "Linemate" => Ok(Resource::Linemate),
            "Mendiane" => Ok(Resource::Mendiane),
            "Nourriture" => Ok(Resource::Nourriture),
            "Phiras" => Ok(Resource::Phiras),
            "Sibur" => Ok(Resource::Sibur),
            "Thystame" => Ok(Resource::Thystame),
            _ => Err(()),
        }
    }
}

impl From<Resource> for usize {
    fn from(value: Resource) -> Self {
        value as u8 as Self
    }
}
