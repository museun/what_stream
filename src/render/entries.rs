use std::io::Write;

use crate::stream::Stream;

use super::{Render, Style, Theme};

pub struct Entries<'a> {
    pub query: &'a str,
    pub streams: &'a [Stream],
}

impl<'a> Render for Entries<'a> {
    fn render(&self, writer: &mut dyn Write, style: &Style, theme: &Theme) -> anyhow::Result<()> {
        use unicode_width::UnicodeWidthStr as _;

        writeln!(
            writer,
            "{left}{query}",
            query = theme.category.paint(&self.query),
            left = theme.fringe.paint(&*style.top)
        )?;

        let title_left_len = style.title.len();

        let max_width = super::width() - title_left_len;
        for (n, stream) in self.streams.iter().enumerate() {
            if n > 0 {
                writeln!(writer, "{}", theme.entry.paint(&*style.entry_sep))?;
            }

            writeln!(
                writer,
                "{left}[{language}] https://twitch.tv/{link}",
                language = theme
                    .spoken_language
                    .paint(&stream.language.to_ascii_uppercase()),
                link = theme.link.paint(&stream.user_name),
                left = theme.fringe.paint(&*style.link),
            )?;

            write!(writer, "{left}", left = theme.fringe.paint(&*style.title))?;

            let title = stream.title.trim();

            // if the title would wrap, partition it. but only if we're printing a left fringe
            if title.width() > max_width && !style.title.is_empty() {
                for word in crate::string::partition_line(title, max_width, title_left_len) {
                    match word {
                        crate::string::LinePartition::Continuation(word) => {
                            write!(writer, "{}", theme.title.paint(word))?
                        }
                        crate::string::LinePartition::Start(word) => {
                            write!(
                                writer,
                                "\n{left}{sp: >title_left_len$}{word}",
                                word = theme.title.paint(word.trim_start()),
                                left = theme.fringe.paint(&*style.continuation),
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

            let print_tags = !stream.user_tag_map.is_empty();
            let end = if n < self.streams.len() - 1 {
                &*style.stats
            } else {
                &*style.end
            };

            writeln!(
                writer,
                "{left}started {uptime} ago, {viewers} watching",
                uptime = theme.uptime.paint(&stream.started_at),
                viewers = theme.viewers.paint(&stream.viewer_count),
                left = theme
                    .fringe
                    .paint(if print_tags { &*style.stats } else { end })
            )?;

            if !print_tags {
                continue;
            }

            write!(writer, "{left}tags: ", left = theme.fringe.paint(end))?;
            let len = stream.user_tag_map.len();
            let mut tags = stream.user_tag_map.values().collect::<Vec<_>>();
            tags.sort_unstable();

            for (i, tags) in tags.into_iter().enumerate() {
                write!(
                    writer,
                    "{}{}",
                    theme.tag.paint(tags),
                    if i < len - 1 { " | " } else { "" }
                )?;
            }
            writeln!(writer)?;
        }

        Ok(())
    }
}
