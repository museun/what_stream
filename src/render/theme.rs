use yansi::{Color, Style};

pub struct Theme {
    pub fringe: Style,
    pub entry: Style,

    pub category: Style,
    pub link: Style,
    pub title: Style,
    pub uptime: Style,
    pub viewers: Style,
}

impl Theme {
    pub fn standard() -> Self {
        Self {
            fringe: Style::new(Color::Black).bold(),
            entry: Style::new(Color::Black).bold(),

            category: Style::new(Color::Magenta),
            link: Style::new(Color::Blue).bold(),
            title: Style::new(Color::Yellow).bold(),
            uptime: Style::new(Color::Green),
            viewers: Style::new(Color::Cyan),
        }
    }
}

// TODO more themes
impl Default for Theme {
    fn default() -> Self {
        Self::standard()
    }
}
