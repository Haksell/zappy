use rand::{seq::SliceRandom as _, thread_rng};
use serde::{Deserialize, Serialize};
use std::fmt::Display;

pub type StoneSet = [usize; Stone::SIZE];

pub trait StoneSetOperations {
    fn reduce_current_from(&mut self, other: &StoneSet) -> bool;
}

impl StoneSetOperations for StoneSet {
    fn reduce_current_from(&mut self, other: &StoneSet) -> bool {
        let has_enough_resources = self.iter().zip(other.iter()).all(|(a, b)| a >= b);
        if has_enough_resources {
            for (idx, count) in other.iter().enumerate() {
                self[idx] -= count;
            }
        }
        has_enough_resources
    }
}

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Hash, Clone, Copy)]
#[repr(u8)]
pub enum Stone {
    Deraumere,
    Linemate,
    Mendiane,
    Phiras,
    Sibur,
    Thystame,
}

impl TryFrom<u8> for Stone {
    type Error = &'static str;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Stone::Deraumere),
            1 => Ok(Stone::Linemate),
            2 => Ok(Stone::Mendiane),
            3 => Ok(Stone::Phiras),
            4 => Ok(Stone::Sibur),
            5 => Ok(Stone::Thystame),
            _ => unreachable!(),
        }
    }
}

impl From<Stone> for usize {
    fn from(value: Stone) -> Self {
        value as u8 as Self
    }
}

impl Stone {
    pub const SIZE: usize = 6; // TODO: dynamic
}

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Clone, Copy)]
pub enum Resource {
    Stone(Stone),
    Nourriture,
}

impl Resource {
    pub const SIZE: usize = Stone::SIZE + 1;

    pub fn alias(&self) -> char {
        match self {
            Resource::Stone(stone) => match stone {
                Stone::Deraumere => 'D',
                Stone::Linemate => 'L',
                Stone::Mendiane => 'M',
                Stone::Phiras => 'P',
                Stone::Sibur => 'S',
                Stone::Thystame => 'T',
            },
            Resource::Nourriture => 'N',
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Resource::Stone(stone) => match stone {
                Stone::Deraumere => "deraumere",
                Stone::Linemate => "linemate",
                Stone::Mendiane => "mendiane",
                Stone::Phiras => "phiras",
                Stone::Sibur => "sibur",
                Stone::Thystame => "thystame",
            },
            Resource::Nourriture => "nourriture",
        }
    }

    pub fn random() -> Self {
        //TODO: nouriture is deleted
        static RESOURCES: [Resource; Stone::SIZE] = [
            Resource::Stone(Stone::Deraumere),
            Resource::Stone(Stone::Linemate),
            Resource::Stone(Stone::Mendiane),
            Resource::Stone(Stone::Phiras),
            Resource::Stone(Stone::Sibur),
            Resource::Stone(Stone::Thystame),
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
            "deraumere" | "d" => Ok(Resource::Stone(Stone::Deraumere)),
            "linemate" | "l" => Ok(Resource::Stone(Stone::Linemate)),
            "mendiane" | "m" => Ok(Resource::Stone(Stone::Mendiane)),
            "phiras" | "p" => Ok(Resource::Stone(Stone::Phiras)),
            "sibur" | "s" => Ok(Resource::Stone(Stone::Sibur)),
            "thystame" | "t" => Ok(Resource::Stone(Stone::Thystame)),
            "nourriture" | "n" => Ok(Resource::Nourriture),
            _ => Err(()),
        }
    }
}

impl TryFrom<usize> for Resource {
    type Error = &'static str;

    fn try_from(value: usize) -> Result<Self, Self::Error> {
        match value {
            0..Stone::SIZE => Ok(Resource::Stone(Stone::try_from(value as u8).unwrap())),
            Stone::SIZE => Ok(Resource::Nourriture),
            _ => unreachable!(),
        }
    }
}

impl From<Resource> for usize {
    fn from(value: Resource) -> Self {
        match value {
            Resource::Stone(stone) => stone as usize,
            Resource::Nourriture => Stone::SIZE,
        }
    }
}
