use std::borrow::Cow;

use crate::{Entries, Render, Stream};

pub struct Demo;

impl Demo {
    #[cfg_attr(debug_assertions, allow(dead_code,))]
    const ENTRIES: [Entries<'static>; 2] = [
        Entries {
            query: "rust",
            streams: &[Stream {
                started_at: Cow::Borrowed("5 minutes"),
                title: Cow::Borrowed("some example title"),
                user_name: Cow::Borrowed("a_rustacean"),
                user_id: Cow::Borrowed("12345"),
                viewer_count: 7,
                language: Cow::Borrowed("en"),
                tag_ids: Cow::Borrowed(&[Cow::Borrowed("rust")]),
                uptime: 0,
            }],
        },
        Entries {
            query: "c++",
            streams: &[
                Stream {
                    started_at: Cow::Borrowed("1 hour 40 minutes"),
                    title: Cow::Borrowed("another title"),
                    user_name: Cow::Borrowed("a_cpp_dev"),
                    user_id: Cow::Borrowed("12346"),
                    viewer_count: 1,
                    language: Cow::Borrowed("en"),
                    tag_ids: Cow::Borrowed(&[Cow::Borrowed("c++")]),
                    uptime: 0,
                },
                Stream {
                    started_at: Cow::Borrowed("25 minutes"),
                    title: Cow::Borrowed("a third title, but this time its a bit longer and it should be used for wrapping the text. but sometimes the terminal is too wide, so lets add more meandering things to increase the word count"),
                    user_name: Cow::Borrowed("some_person"),
                    user_id: Cow::Borrowed("12347"),
                    viewer_count: 2,
                    language: Cow::Borrowed("fr"),
                    tag_ids: Cow::Borrowed(&[Cow::Borrowed("")]),
                    uptime: 0,
                },
            ],
        },
    ];

    pub fn show_off(
        writer: &mut dyn std::io::Write,
        style: &crate::Style,
        theme: &crate::Theme,
    ) -> anyhow::Result<()> {
        std::array::IntoIter::new(Self::ENTRIES)
            .enumerate()
            .try_for_each(|(n, entries)| {
                if n > 0 {
                    writeln!(writer)?;
                }
                entries.render(writer, style, theme)
            })
    }
}
