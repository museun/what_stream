use std::io::Write;

use crate::stream::Stream;

use super::{Render, Style, Theme};

pub struct Entries<'a> {
    pub category: &'a str,
    pub streams: &'a [Stream],
}

impl<'a> Render for Entries<'a> {
    fn render(&self, writer: &mut dyn Write, style: &Style, theme: &Theme) -> anyhow::Result<()> {
        use unicode_width::UnicodeWidthStr as _;

        writeln!(
            writer,
            "{left}{category}",
            category = theme.category.paint(&self.category),
            left = theme.fringe.paint(style.top)
        )?;

        let title_left_len = style.title.len();

        let max_width = super::width() - title_left_len;
        for (n, stream) in self.streams.iter().enumerate() {
            if n > 0 {
                writeln!(writer, "{}", theme.entry.paint(&style.entry_sep))?;
            }

            writeln!(
                writer,
                "{left}https://twitch.tv/{link}",
                link = theme.link.paint(&stream.user_name),
                left = theme.fringe.paint(&style.link),
            )?;

            write!(writer, "{left}", left = theme.fringe.paint(style.title))?;

            // if the title would wrap, partition it. but only if we're printing a left fringe
            if stream.title.width() > max_width && !style.title.is_empty() {
                for word in crate::string::partition_line(&*stream.title, max_width, title_left_len)
                {
                    match word {
                        crate::string::LinePartition::Continuation(word) => {
                            write!(writer, "{}", theme.title.paint(word))?
                        }
                        crate::string::LinePartition::Start(word) => {
                            write!(
                                writer,
                                "\n{left}{sp: >title_left_len$}{word}",
                                word = theme.title.paint(word.trim_start()),
                                left = theme.fringe.paint(style.continuation),
                                title_left_len = title_left_len - style.title.len(),
                                sp = ""
                            )?;
                        }
                    }
                }
                writeln!(writer)?;
            } else {
                // otherwise just write the title
                writeln!(writer, "{}", theme.title.paint(&stream.title))?;
            }

            writeln!(
                writer,
                "{left}started {uptime} ago, {viewers} watching",
                uptime = theme.uptime.paint(&stream.started_at),
                viewers = theme.viewers.paint(&stream.viewer_count),
                left = theme.fringe.paint(if n < self.streams.len() - 1 {
                    style.stats
                } else {
                    style.end
                })
            )?;
        }

        Ok(())
    }
}
