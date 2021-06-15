use super::{Color, ColorStyle};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Theme {
    pub fringe: ColorStyle,
    pub entry: ColorStyle,

    pub category: ColorStyle,
    pub spoken_language: ColorStyle,
    pub link: ColorStyle,
    pub title: ColorStyle,
    pub uptime: ColorStyle,
    pub viewers: ColorStyle,
}

impl Theme {
    pub fn standard() -> Self {
        Self {
            fringe: ColorStyle::new(Color(128, 128, 128)).bold(),
            entry: ColorStyle::new(Color(96, 96, 96)).bold(),

            category: ColorStyle::new(Color::MAGENTA),
            spoken_language: ColorStyle::new(Color::BRIGHT_WHITE),
            link: ColorStyle::new(Color::BRIGHT_BLUE).bold(),
            title: ColorStyle::new(Color::YELLOW).bold(),
            uptime: ColorStyle::new(Color::GREEN),
            viewers: ColorStyle::new(Color::CYAN),
        }
    }
}

impl Default for Theme {
    fn default() -> Self {
        Self::standard()
    }
}
