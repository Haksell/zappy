pub type RGB = (u8, u8, u8);

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
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
    pub fn _to_rgb(&self) -> (u8, u8, u8) {
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
}
