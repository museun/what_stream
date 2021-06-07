use std::{borrow::Cow, fmt::Display};

use serde::Serializer;

#[derive(Copy, Clone)]
pub struct Color(pub u8, pub u8, pub u8);

impl Color {
    pub const BLACK: Self = Self(12, 12, 12);
    pub const RED: Self = Self(197, 15, 31);
    pub const GREEN: Self = Self(19, 161, 14);
    pub const YELLOW: Self = Self(193, 156, 0);
    pub const BLUE: Self = Self(0, 55, 218);
    pub const MAGENTA: Self = Self(136, 23, 152);
    pub const CYAN: Self = Self(58, 150, 221);
    pub const WHITE: Self = Self(204, 204, 204);
    pub const BRIGHT_BLACK: Self = Self(118, 118, 118);
    pub const BRIGHT_RED: Self = Self(231, 72, 86);
    pub const BRIGHT_GREEN: Self = Self(22, 198, 12);
    pub const BRIGHT_YELLOW: Self = Self(249, 241, 165);
    pub const BRIGHT_BLUE: Self = Self(59, 120, 255);
    pub const BRIGHT_MAGENTA: Self = Self(180, 0, 158);
    pub const BRIGHT_CYAN: Self = Self(97, 214, 214);
    pub const BRIGHT_WHITE: Self = Self(242, 242, 242);

    pub const fn default_fg() -> Self {
        Self(0xC0, 0xC0, 0xC0)
    }

    pub const fn default_bg() -> Self {
        Self(0x00, 0x00, 0x00)
    }
}

impl Default for Color {
    fn default() -> Self {
        Self::default_fg()
    }
}

impl std::str::FromStr for Color {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        use anyhow::Context as _;

        let s = s.trim();
        let s = match s.len() {
            7 if s.starts_with('#') => &s[1..],
            6 if s.chars().all(|c| c.is_ascii_hexdigit()) => s,
            _ => anyhow::bail!("invalid hex string"),
        };

        u32::from_str_radix(s, 16)
            .map(|s| {
                Self(
                    ((s >> 16) & 0xFF) as _,
                    ((s >> 8) & 0xFF) as _,
                    (s & 0xFF) as _,
                )
            })
            .with_context(|| "cannot parse hex string")
    }
}

impl std::fmt::Debug for Color {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let Self(r, g, b) = self;
        write!(f, "#{:02X}{:02X}{:02X}", r, g, b)
    }
}

impl serde::Serialize for Color {
    fn serialize<S>(&self, ser: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let Self(r, g, b) = self;
        ser.collect_str(&format_args!("#{:02X}{:02X}{:02X}", r, g, b))
    }
}

impl<'de> serde::Deserialize<'de> for Color {
    fn deserialize<D>(de: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        <Cow<'de, str>>::deserialize(de)?
            .parse()
            .map_err(|_| serde::de::Error::custom("invalid hex string"))
    }
}

#[derive(Copy, Clone, Debug, serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct ColorStyle {
    pub fg: Color,
    pub bg: Option<Color>,
    pub bold: bool,
}

impl ColorStyle {
    pub fn new(fg: Color) -> Self {
        Self {
            fg,
            ..Default::default()
        }
    }

    pub const fn bold(self) -> Self {
        Self {
            bold: !self.bold,
            ..self
        }
    }

    pub fn paint<T>(self, item: T) -> yansi::Paint<T>
    where
        T: Display,
    {
        let Color(r, g, b) = self.fg;
        let mut p = yansi::Paint::rgb(r, g, b, item);
        if let Some(Color(r, g, b)) = self.bg {
            p = p.bg(yansi::Color::RGB(r, g, b));
        }
        if self.bold {
            p = p.bold();
        }
        p
    }
}

impl Default for ColorStyle {
    fn default() -> Self {
        Self {
            fg: Color::default_fg(),
            bg: None,
            bold: false,
        }
    }
}
