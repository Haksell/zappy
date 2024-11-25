use rand::{seq::SliceRandom as _, thread_rng};
use serde::{Deserialize, Serialize};
use std::fmt::Display;

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Hash, Clone, Copy)]
#[repr(u8)]
pub enum Mining {
    Deraumere,
    Linemate,
    Mendiane,
    Phiras,
    Sibur,
    Thystame,
}

impl TryFrom<u8> for Mining {
    type Error = &'static str;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Mining::Deraumere),
            1 => Ok(Mining::Linemate),
            2 => Ok(Mining::Mendiane),
            3 => Ok(Mining::Phiras),
            4 => Ok(Mining::Sibur),
            5 => Ok(Mining::Thystame),
            _ => unreachable!(),
        }
    }
}

impl From<Mining> for usize {
    fn from(value: Mining) -> Self {
        value as u8 as Self
    }
}

impl Mining {
    pub const SIZE: usize = 6; // TODO: dynamic
}

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Hash, Clone, Copy)]
pub enum Resource {
    Mining(Mining),
    Nourriture,
}

impl Resource {
    pub const SIZE: usize = Mining::SIZE + 1;

    pub fn alias(&self) -> char {
        match self {
            Resource::Mining(mining) => match mining {
                Mining::Deraumere => 'D',
                Mining::Linemate => 'L',
                Mining::Mendiane => 'M',
                Mining::Phiras => 'P',
                Mining::Sibur => 'S',
                Mining::Thystame => 'T',
            },
            Resource::Nourriture => 'N',
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Resource::Mining(mining) => match mining {
                Mining::Deraumere => "Deraumere",
                Mining::Linemate => "Linemate",
                Mining::Mendiane => "Mendiane",
                Mining::Phiras => "Phiras",
                Mining::Sibur => "Sibur",
                Mining::Thystame => "Thystame",
            },
            Resource::Nourriture => "Nourriture",
        }
    }

    pub fn random() -> Self {
        static RESOURCES: [Resource; Resource::SIZE] = [
            Resource::Mining(Mining::Deraumere),
            Resource::Mining(Mining::Linemate),
            Resource::Mining(Mining::Mendiane),
            Resource::Mining(Mining::Phiras),
            Resource::Mining(Mining::Sibur),
            Resource::Mining(Mining::Thystame),
            Resource::Nourriture,
        ];

        let mut rng = thread_rng();
        *RESOURCES.choose(&mut rng).unwrap()
    }
}

impl Display for Resource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl TryFrom<&str> for Resource {
    type Error = ();

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value.to_ascii_lowercase().as_str() {
            "deraumere" | "d" => Ok(Resource::Mining(Mining::Deraumere)),
            "linemate" | "l" => Ok(Resource::Mining(Mining::Linemate)),
            "mendiane" | "m" => Ok(Resource::Mining(Mining::Mendiane)),
            "phiras" | "p" => Ok(Resource::Mining(Mining::Phiras)),
            "sibur" | "s" => Ok(Resource::Mining(Mining::Sibur)),
            "thystame" | "t" => Ok(Resource::Mining(Mining::Thystame)),
            "nourriture" | "n" => Ok(Resource::Nourriture),
            _ => Err(()),
        }
    }
}

impl TryFrom<usize> for Resource {
    type Error = &'static str;

    fn try_from(value: usize) -> Result<Self, Self::Error> {
        match value {
            0..Mining::SIZE => Ok(Resource::Mining(Mining::try_from(value as u8).unwrap())),
            Mining::SIZE => Ok(Resource::Nourriture),
            _ => unreachable!(),
        }
    }
}

impl From<Resource> for usize {
    fn from(value: Resource) -> Self {
        match value {
            Resource::Mining(mining) => mining as usize,
            Resource::Nourriture => Mining::SIZE,
        }
    }
}
