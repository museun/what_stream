#![cfg_attr(debug_assertions, allow(dead_code,))]

use anyhow::Context;
use std::collections::HashMap;
use std::io::Write;

fn trim_word_boundaries(s: &str) -> &str {
    const HEAD: &str = "([{";
    const TAIL: &str = ",.!?-:}])";
    s.trim_start_matches(|c| HEAD.contains(c))
        .trim_end_matches(|c| TAIL.contains(c))
        .trim()
}

#[derive(Debug, Copy, Clone)]
struct SortAction {
    column: Column,
    direction: Direction,
}

impl std::str::FromStr for SortAction {
    type Err = anyhow::Error;
    fn from_str(flag: &str) -> anyhow::Result<Self> {
        let mut iter = flag.splitn(2, ',');
        let head = iter.next().with_context(|| "a column must be provided")?;
        let column = match head {
            "viewers" => Column::Viewers,
            "uptime" => Column::Uptime,
            "name" => Column::Name,
            name => anyhow::bail!("invalid column: {}", name),
        };

        let direction = iter
            .next()
            .map(|tail| match tail {
                "asc" | "ascending" => Ok(Direction::Ascending),
                "desc" | "descending" | "" => Ok(Direction::Descending),
                dir => anyhow::bail!("invalid direction: {}", dir),
            })
            .transpose()?
            .unwrap_or(Direction::Descending);

        Ok(Self { column, direction })
    }
}

#[derive(Debug, Copy, Clone, PartialOrd, PartialEq, Ord, Eq)]
enum Column {
    Viewers,
    Uptime,
    Name,
}

#[derive(Debug, Copy, Clone, PartialOrd, PartialEq)]
enum Direction {
    Descending,
    Ascending,
}

#[derive(Debug)]
struct Args {
    sort: Option<SortAction>,
    query: Vec<String>,
    json: bool,
    style: Style,
}

impl Args {
    fn parse() -> anyhow::Result<Self> {
        let mut args = pico_args::Arguments::from_env();

        if args.contains("-h") {
            Self::print_short_help();
            std::process::exit(0);
        }
        if args.contains("--help") {
            Self::print_long_help();
            std::process::exit(0);
        }
        if args.contains(["-v", "--version"]) {
            Self::print_version();
            std::process::exit(0);
        }

        let json = args.contains(["-j", "--json"]);
        let sort = args.opt_value_from_str(["-s", "--sort"])?;

        let style = args
            .opt_value_from_fn(["-t", "--style"], |s| match s {
                "fancy" => Ok(Style::FANCY),
                "box" => Ok(Style::BOX),
                "none" => Ok(Style::NONE),
                s => anyhow::bail!("unknown style: {}", s),
            })?
            .unwrap_or(Style::BOX);

        let query = args.free()?;
        Ok(Self {
            sort,
            query,
            json,
            style,
        })
    }

    fn print_short_help() {
        Self::print_version();
        println!();
        println!("{}", include_str!("../short_help.txt"));
    }

    fn print_long_help() {
        Self::print_version();
        println!();
        println!("{}", include_str!("../short_help.txt"));
        println!("{}", include_str!("../long_help.txt"));
    }

    fn print_version() {
        println!("{} {}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));
    }
}

struct Secrets {
    client_id: String,
    bearer_oauth: String,
}

impl Secrets {
    fn get() -> anyhow::Result<Self> {
        let client_id = std::env::var("WHAT_STREAM_CLIENT_ID").unwrap_or_else(|_| {
            eprintln!("please set 'WHAT_STREAM_CLIENT_ID' to your Twitch Client ID");
            std::process::exit(1)
        });
        let bearer_oauth = std::env::var("WHAT_STREAM_BEARER_OAUTH").unwrap_or_else(|_| {
            eprintln!("please set 'WHAT_STREAM_BEARER_OAUTH' to your Twitch Bearer OAuth token");
            std::process::exit(1)
        });

        Ok(Self {
            client_id,
            bearer_oauth,
        })
    }
}

#[derive(serde::Deserialize, serde::Serialize)]
struct Stream {
    started_at: String,
    title: String,
    user_name: String,
    viewer_count: i64,

    #[allow(dead_code)]
    language: String,

    #[serde(skip)]
    uptime: i64,
}

fn fetch_streams<'a>(
    query: &'a [String],
    Secrets {
        client_id,
        bearer_oauth,
    }: &Secrets,
) -> anyhow::Result<Vec<(&'a String, Stream)>> {
    #[derive(serde::Deserialize)]
    struct Resp<T> {
        data: Vec<T>,
        pagination: Pagination,
    }

    #[derive(Default, serde::Deserialize)]
    struct Pagination {
        #[serde(default)]
        cursor: String,
    }

    let mut after = String::new();
    let mut streams = std::iter::from_fn(|| {
        let resp: Resp<Stream> = attohttpc::get("https://api.twitch.tv/helix/streams")
            .param("game_id", "509670") // TODO this is hardcoded (for 'science and technology')
            .param("first", "100")
            .param("after", &after)
            .header("client-id", client_id)
            .bearer_auth(bearer_oauth)
            .send()
            .ok()?
            .json()
            .ok()?;

        match resp.data.is_empty() {
            true => None,
            false => {
                after = resp.pagination.cursor;
                Some(resp.data)
            }
        }
    })
    .flatten()
    .filter_map(|stream| {
        for part in stream
            .title
            .split(' ')
            .map(trim_word_boundaries)
            .filter(|s| !s.is_empty())
            .map(|s| s.to_lowercase())
        {
            for q in query {
                if *q == part {
                    return Some((q, stream));
                }
            }
        }
        None
    })
    .collect::<Vec<_>>();

    streams.iter_mut().for_each(|(_, stream)| {
        let (date, time_) = {
            let mut i = stream.started_at.split('T');
            (i.next().unwrap(), i.next().unwrap())
        };

        let duration = time::OffsetDateTime::now_utc()
            - time::OffsetDateTime::parse(
                format!("{} {} +0000", date, &time_[..time_.len() - 1]),
                "%F %T %z",
            )
            .unwrap();

        // TODO do this do differently
        let seconds = duration.whole_seconds();
        let hours = (seconds / 60) / 60;
        let minutes = (seconds / 60) % 60;

        let started = if hours > 0 {
            format!(
                "{hours} hours {minutes} minutes",
                hours = hours,
                minutes = minutes
            )
        } else {
            format!("{minutes} minutes", minutes = minutes)
        };

        stream.uptime = seconds;
        stream.started_at = started;
    });

    Ok(streams)
}

fn sort_streams(streams: &mut Vec<Stream>, option: Option<SortAction>) {
    use {Column::*, Direction::*};
    streams.sort_unstable_by(|left, right| {
        option
            .map(|sort| {
                let SortAction { column, direction } = sort;
                let ordering = match column {
                    Viewers => left.viewer_count.cmp(&right.viewer_count),
                    Uptime => left.uptime.cmp(&right.uptime),
                    // invert this so its a->z not z->a
                    Name => right.user_name.cmp(&left.user_name),
                    _ => unreachable!(),
                };

                match direction {
                    Ascending => ordering,
                    Descending => ordering.reverse(),
                }
            })
            .unwrap_or_else(|| left.viewer_count.cmp(&right.viewer_count))
    });
}

trait Render {
    fn render(&self, writer: &mut dyn Write, style: &Style, theme: &Theme) -> anyhow::Result<()>;
}

#[derive(Copy, Clone, Debug)]
struct Style {
    top: &'static str,
    entry_sep: &'static str,
    end: &'static str,

    link: &'static str,

    title: &'static str,
    continuation: &'static str,

    uptime: &'static str,
    viewers: &'static str,
}

impl Style {
    const NONE: Style = Style {
        top: "",
        entry_sep: "",
        end: "",

        link: "",

        title: "",
        continuation: "",

        uptime: "",
        viewers: "",
    };

    const BOX: Style = Style {
        top: "┌── ",
        entry_sep: "│",
        end: "└ ",

        link: "├ ",

        title: "├ ",
        continuation: "│ ",

        uptime: "├ ",
        viewers: "├ ",
    };

    const FANCY: Style = Style {
        top: "╭╍╍ ",
        entry_sep: "╎",
        end: "╰╍ ",

        link: "╞═ ",

        title: "├┄ ",
        continuation: "┆ ",

        uptime: "├┄ ",
        viewers: "├┄ ",
    };
}

enum LinePartition<'a> {
    Start(&'a str),
    Continuation(&'a str),
}

fn partition_line(
    input: &str,
    max: usize,
    left: usize,
) -> impl Iterator<Item = LinePartition<'_>> + '_ {
    use {
        unicode_segmentation::UnicodeSegmentation as _, //
        unicode_width::UnicodeWidthStr as _,
    };
    let mut budget = max;
    input.split_word_bounds().map(move |word| {
        let width = word.width();
        match budget.checked_sub(width) {
            Some(n) => {
                budget = n;
                LinePartition::Continuation(word)
            }
            None => {
                budget = max - width - left;
                LinePartition::Start(word)
            }
        }
    })
}

use yansi::Style as ColorStyle;

struct Theme {
    fringe: ColorStyle,
    entry: ColorStyle,

    category: ColorStyle,
    link: ColorStyle,
    title: ColorStyle,
    uptime: ColorStyle,
    viewers: ColorStyle,
}

// TODO more themes
impl Default for Theme {
    fn default() -> Self {
        use yansi::Color;
        Self {
            fringe: ColorStyle::new(Color::Black).bold(),
            entry: ColorStyle::new(Color::Black).bold(),

            category: ColorStyle::new(Color::Magenta),
            link: ColorStyle::new(Color::Blue).bold(),
            title: ColorStyle::new(Color::Yellow).bold(),
            uptime: ColorStyle::new(Color::Green),
            viewers: ColorStyle::new(Color::Cyan),
        }
    }
}

// TODO other renderers

struct Entries<'a> {
    category: &'a str,
    streams: &'a [Stream],
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

        let max_width = width() - title_left_len;
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
                for word in partition_line(&*stream.title, max_width, title_left_len) {
                    match word {
                        LinePartition::Continuation(word) => {
                            write!(writer, "{}", theme.title.paint(word))?
                        }
                        LinePartition::Start(word) => {
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
                    style.viewers
                } else {
                    style.end
                })
            )?;
        }

        Ok(())
    }
}

fn width() -> usize {
    terminal_size::terminal_size()
        .map(|(terminal_size::Width(width), _)| width)
        .map(usize::from)
        .unwrap_or(40)
}

fn try_enable_colors() {
    if std::env::var("NO_COLOR").is_ok() || (cfg!(windows) && !yansi::Paint::enable_windows_ascii())
    {
        yansi::Paint::disable();
    } else {
        yansi::Paint::enable();
    }
}

fn render_streams<'a>(
    out: &mut dyn Write,
    style: &Style,
    theme: &Theme,
    streams: impl IntoIterator<Item = (String, &'a [Stream])>,
) -> anyhow::Result<()> {
    streams
        .into_iter()
        .enumerate()
        .try_for_each(|(n, (category, streams))| {
            if n > 0 {
                writeln!(out)?;
            }
            Entries {
                category: &category,
                streams: &streams,
            }
            .render(out, style, theme)
        })
}

fn main() -> anyhow::Result<()> {
    let secrets = Secrets::get()?;
    let args = Args::parse()?;

    if args.query.is_empty() {
        eprintln!("please provide something to filter by");
        std::process::exit(1)
    }

    let mut streams = fetch_streams(&args.query, &secrets)?.into_iter().fold(
        HashMap::<_, Vec<_>>::new(),
        |mut map, (category, stream)| {
            map.entry(category.clone()).or_default().push(stream);
            map
        },
    );

    if args.json {
        println!("{}", serde_json::to_string_pretty(&streams)?);
        std::process::exit(0)
    }

    for streams in streams.values_mut() {
        sort_streams(streams, args.sort)
    }

    try_enable_colors();
    let out = std::io::stdout();
    let mut out = out.lock();

    let streams = args
        .query
        .into_iter()
        .filter_map(|q| streams.get(&*q).map(|s| (q, s.as_slice())));

    render_streams(&mut out, &args.style, &Theme::default(), streams)
}
