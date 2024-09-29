#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Color {
    Red,
    Green,
    Blue,
    Yellow,
    Magenta,
    Cyan,
    White,
    Black,
    BrightGreen,
    BrightRed,
}

impl Color {
    /// Returns the ANSI escape code for the color.
    pub fn ansi_code(&self) -> String {
        match self {
            Color::Red => "\x1b[31m".to_string(),
            Color::Green => "\x1b[32m".to_string(),
            Color::Blue => "\x1b[34m".to_string(),
            Color::Yellow => "\x1b[33m".to_string(),
            Color::Magenta => "\x1b[35m".to_string(),
            Color::Cyan => "\x1b[36m".to_string(),
            Color::White => "\x1b[37m".to_string(),
            Color::Black => "\x1b[30m".to_string(),
            Color::BrightGreen => "\x1b[92m".to_string(),
            Color::BrightRed => "\x1b[91m".to_string(),
        }
    }

    pub fn from_str(s: &str) -> Option<Color> {
        match s.to_lowercase().as_str() {
            "red" => Some(Color::Red),
            "green" => Some(Color::Green),
            "blue" => Some(Color::Blue),
            "yellow" => Some(Color::Yellow),
            "magenta" => Some(Color::Magenta),
            "cyan" => Some(Color::Cyan),
            "white" => Some(Color::White),
            "black" => Some(Color::Black),
            "bright_green" => Some(Color::BrightGreen),
            "bright_red" => Some(Color::BrightRed),
            _ => None,
        }
    }
}
