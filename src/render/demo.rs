use crate::{Entries, Render, Stream};

pub struct Demo;

impl Demo {
    pub fn show_off(
        writer: &mut dyn std::io::Write,
        style: &crate::Style,
        theme: &crate::Theme,
    ) -> anyhow::Result<()> {
        [
            Entries {
                query: "rust",
                streams: &[Stream {
                    started_at: Box::from("5 minutes"),
                    title: Box::from("some example title"),
                    user_name: Box::from("a_rustacean"),
                    user_id: Box::from("12345"),
                    viewer_count: 7,
                    language: Box::from("en"),
                    tag_ids: Box::from([Box::from("rust")]),
                    uptime: 0,
                }],
            },
            Entries {
                query: "c++",
                streams: &[
                    Stream {
                        started_at: Box::from("1 hour 40 minutes"),
                        title: Box::from("another title"),
                        user_name: Box::from("a_cpp_dev"),
                        user_id: Box::from("12346"),
                        viewer_count: 1,
                        language: Box::from("en"),
                        tag_ids: Box::from([Box::from("c++")]),
                        uptime: 0,
                    },
                    Stream {
                        started_at: Box::from("25 minutes"),
                        title: Box::from("a third title, but this time its a bit longer and it should be used for wrapping the text. but sometimes the terminal is too wide, so lets add more meandering things to increase the word count"),
                        user_name: Box::from("some_person"),
                        user_id: Box::from("12347"),
                        viewer_count: 2,
                        language: Box::from("fr"),
                        tag_ids: Box::from([Box::from("")]),
                        uptime: 0,
                    },
                ],
            },
        ]
            .into_iter()
            .enumerate()
            .try_for_each(|(n, entries)| {
                if n > 0 {
                    writeln!(writer)?;
                }
                entries.render(writer, style, theme)
            })
    }
}
