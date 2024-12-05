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
    pub fn to_ratatui_value(&self) -> RatatuiColor {
        match self {
            ZappyColor::Red => RatatuiColor::Red,
            ZappyColor::Green => RatatuiColor::Green,
            ZappyColor::Yellow => RatatuiColor::Yellow,
            ZappyColor::Blue => RatatuiColor::Blue,
            ZappyColor::Magenta => RatatuiColor::Magenta,
            ZappyColor::Cyan => RatatuiColor::Cyan,
            ZappyColor::Gray => RatatuiColor::Gray,
            ZappyColor::DarkGray => RatatuiColor::DarkGray,
            ZappyColor::LightRed => RatatuiColor::LightRed,
            ZappyColor::LightGreen => RatatuiColor::LightGreen,
            ZappyColor::LightYellow => RatatuiColor::LightYellow,
            ZappyColor::LightBlue => RatatuiColor::LightBlue,
            ZappyColor::LightMagenta => RatatuiColor::LightMagenta,
            ZappyColor::LightCyan => RatatuiColor::LightCyan,
        }
    }

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
