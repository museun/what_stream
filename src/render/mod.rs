use std::io::Write;

mod color;
pub use color::{Color, ColorStyle};

mod theme;
pub use theme::Theme;

mod style;
pub use style::Style;

mod entries;
pub use entries::Entries;

mod demo;
pub use demo::Demo;

pub trait Render {
    fn render(&self, writer: &mut dyn Write, style: &Style, theme: &Theme) -> anyhow::Result<()>;
}

fn width() -> usize {
    terminal_size::terminal_size()
        .map(|(terminal_size::Width(width), _)| width)
        .map(usize::from)
        .unwrap_or(40)
}
