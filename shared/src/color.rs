use serde::{Deserialize, Serialize};

pub type RGB = (u8, u8, u8);

#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ZappyColor {
    Red,
    Green,
    Yellow,
    Blue,
    Magenta,
    Cyan,
    Gray,
    DarkGray,
    LightRed,
    LightGreen,
    LightYellow,
    LightBlue,
    LightMagenta,
    LightCyan,
}

pub const ZAPPY_COLORS: [ZappyColor; 14] = [
    ZappyColor::Red,
    ZappyColor::Green,
    ZappyColor::Yellow,
    ZappyColor::Blue,
    ZappyColor::Magenta,
    ZappyColor::Cyan,
    ZappyColor::Gray,
    ZappyColor::DarkGray,
    ZappyColor::LightRed,
    ZappyColor::LightGreen,
    ZappyColor::LightYellow,
    ZappyColor::LightBlue,
    ZappyColor::LightMagenta,
    ZappyColor::LightCyan,
];
