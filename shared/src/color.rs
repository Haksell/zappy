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

impl ZappyColor {
    const COLORS: [ZappyColor; 14] = [
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

    pub fn rgb(&self) -> (u8, u8, u8) {
        match self {
            ZappyColor::Red => (255, 0, 0),
            ZappyColor::Green => (0, 255, 0),
            ZappyColor::Yellow => (255, 255, 0),
            ZappyColor::Blue => (0, 0, 255),
            ZappyColor::Magenta => (255, 0, 255),
            ZappyColor::Cyan => (0, 255, 255),
            ZappyColor::Gray => (128, 128, 128),
            ZappyColor::DarkGray => (64, 64, 64),
            ZappyColor::LightRed => (255, 128, 128),
            ZappyColor::LightGreen => (128, 255, 128),
            ZappyColor::LightYellow => (255, 255, 128),
            ZappyColor::LightBlue => (128, 128, 255),
            ZappyColor::LightMagenta => (255, 128, 255),
            ZappyColor::LightCyan => (128, 255, 255),
        }
    }

    pub fn idx(color_idx: usize) -> Self {
        Self::COLORS[color_idx % Self::COLORS.len()]
    }
}
