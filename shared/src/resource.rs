use crate::color::ZappyColor;
use rand::{seq::SliceRandom as _, thread_rng};
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};

pub type StoneSet = [usize; Stone::SIZE];
pub const RESOURCE_PROPORTION: f32 = 0.06;

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
        match self {
            Stone::Deraumere => ZappyColor::Red,
            Stone::Linemate => ZappyColor::Green,
            Stone::Mendiane => ZappyColor::Yellow,
            Stone::Phiras => ZappyColor::Blue,
            Stone::Sibur => ZappyColor::Magenta,
            Stone::Thystame => ZappyColor::Cyan,
        }
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

    pub fn color(&self) -> ZappyColor {
        match self {
            Resource::Stone(stone) => stone.color(),
            Resource::Nourriture => NOURRITURE_COLOR,
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
