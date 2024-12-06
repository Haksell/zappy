use rand::{seq::SliceRandom as _, thread_rng};
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};

use crate::color::ZappyColor;

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

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Clone, Copy, Hash)]
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

    pub fn index(self) -> usize {
        self as usize
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Stone::Deraumere => "deraumere",
            Stone::Linemate => "linemate",
            Stone::Mendiane => "mendiane",
            Stone::Phiras => "phiras",
            Stone::Sibur => "sibur",
            Stone::Thystame => "thystame",
        }
    }

    pub fn color(&self) -> ZappyColor {
        ZappyColor::idx(*self as usize)
    }
}

pub const NOURRITURE_COLOR: ZappyColor = ZappyColor::LightMagenta;

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Clone, Copy, Hash)]
pub enum Resource {
    Stone(Stone),
    Nourriture,
}

impl Resource {
    pub const SIZE: usize = Stone::SIZE + 1;

    pub fn alias(&self) -> char {
        match self {
            Resource::Stone(Stone::Deraumere) => 'D',
            Resource::Stone(Stone::Linemate) => 'L',
            Resource::Stone(Stone::Mendiane) => 'M',
            Resource::Stone(Stone::Phiras) => 'P',
            Resource::Stone(Stone::Sibur) => 'S',
            Resource::Stone(Stone::Thystame) => 'T',
            Resource::Nourriture => 'N',
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Resource::Stone(stone) => stone.as_str(),
            Resource::Nourriture => "nourriture",
        }
    }

    pub fn random() -> Self {
        static RESOURCES_WEIGHTS: &[Resource] = &[
            Resource::Stone(Stone::Deraumere),
            Resource::Stone(Stone::Linemate),
            Resource::Stone(Stone::Mendiane),
            Resource::Stone(Stone::Phiras),
            Resource::Stone(Stone::Sibur),
            Resource::Stone(Stone::Thystame),
            Resource::Nourriture,
            Resource::Nourriture,
        ];

        let mut rng = thread_rng();
        *RESOURCES_WEIGHTS.choose(&mut rng).unwrap()
    }

    pub fn cell_position(&self) -> (f32, f32) {
        match self {
            Resource::Stone(Stone::Deraumere) => (0.15, 0.15),
            Resource::Stone(Stone::Linemate) => (0.5, 0.15),
            Resource::Stone(Stone::Mendiane) => (0.85, 0.15),
            Resource::Stone(Stone::Phiras) => (0.15, 0.85),
            Resource::Stone(Stone::Sibur) => (0.5, 0.85),
            Resource::Stone(Stone::Thystame) => (0.85, 0.85),
            Resource::Nourriture => (0.5, 0.5),
        }
    }
}

impl Display for Resource {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl Display for Stone {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
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
            0..Stone::SIZE => Ok(Resource::Stone(Stone::try_from(value as u8)?)),
            _ => Ok(Resource::Nourriture),
        }
    }
}
