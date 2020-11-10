use anyhow::Context;
use std::collections::HashMap;

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

const fn count_digits(d: u64) -> usize {
    let (mut len, mut n) = (1, 1u64);
    while len < 20 {
        n *= 10;
        if n > d {
            return len;
        }
        len += 1;
    }
    len
}

fn trim_word_boundaries(s: &str) -> &str {
    const HEAD: &str = "([{";
    const TAIL: &str = ",.!?-:}])";
    s.trim_start_matches(|c| HEAD.contains(c))
        .trim_end_matches(|c| TAIL.contains(c))
        .trim()
}

#[derive(Debug)]
struct SortAction {
    column: Column,
    direction: Direction,
}

#[derive(Debug, Copy, Clone, PartialOrd, PartialEq, Ord, Eq)]
enum Column {
    Viewers,
    Uptime,
    Name,
    Title,
}

#[derive(Debug, Copy, Clone, PartialOrd, PartialEq)]
enum Direction {
    Descending,
    Ascending,
}

fn parse_sort_flag(flag: &str) -> anyhow::Result<SortAction> {
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

    Ok(SortAction { column, direction })
}

#[derive(Debug)]
struct Args {
    sort: Option<SortAction>,
    visible_columns: Vec<Column>,
    query: Vec<String>,
    json: bool,
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
        let sort = args.opt_value_from_fn(["-s", "--sort"], parse_sort_flag)?;
        let mut columns = args.values_from_fn(["-c", "--column"], |s| {
            Ok(match s {
                "viewers" => Column::Viewers,
                "uptime" => Column::Uptime,
                "name" => Column::Name,
                "title" => Column::Title,
                name => anyhow::bail!("invalid column: {}", name),
            })
        })?;

        columns.dedup();
        columns.sort();

        let visible_columns = if columns.is_empty() {
            vec![Column::Viewers, Column::Uptime, Column::Name, Column::Title]
        } else {
            columns
        };

        let query = args.free()?;
        Ok(Self {
            sort,
            query,
            json,
            visible_columns,
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

fn get_secrets() -> anyhow::Result<Secrets> {
    let client_id = std::env::var("WHAT_STREAM_CLIENT_ID").unwrap_or_else(|_| {
        eprintln!("please set 'WHAT_STREAM_CLIENT_ID' to your Twitch Client ID");
        std::process::exit(1)
    });
    let bearer_oauth = std::env::var("WHAT_STREAM_BEARER_OAUTH").unwrap_or_else(|_| {
        eprintln!("please set 'WHAT_STREAM_BEARER_OAUTH' to your Twitch Bearer OAuth token");
        std::process::exit(1)
    });

    Ok(Secrets {
        client_id,
        bearer_oauth,
    })
}

fn fetch_streams<'a>(
    query: &'a [String],
    Secrets {
        client_id,
        bearer_oauth,
    }: &Secrets,
) -> anyhow::Result<Vec<(&'a String, Stream)>> {
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

        let d = time::OffsetDateTime::now_utc()
            - time::OffsetDateTime::parse(
                format!("{} {} +0000", date, &time_[..time_.len() - 1]),
                "%F %T %z",
            )
            .unwrap();

        let h = (d.whole_seconds() / 60) / 60;
        let m = (d.whole_seconds() / 60) % 60;

        let started = if h > 0 {
            format!("{h}h {m}m", h = h, m = m)
        } else {
            format!("{m}m", m = m)
        };

        stream.uptime = d.whole_seconds();
        stream.started_at = started;
    });

    Ok(streams)
}

fn main() -> anyhow::Result<()> {
    let secrets = get_secrets()?;
    let args = Args::parse()?;

    if args.query.is_empty() {
        // TODO we could just print /all/ streams
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
        let s = serde_json::to_string_pretty(&streams)?;
        println!("{}", s);
        std::process::exit(0)
    }

    for streams in streams.values_mut() {
        streams.sort_unstable_by(|left, right| {
            use {Column::*, Direction::*};

            args.sort
                .as_ref()
                .map(|sort| {
                    let &SortAction { column, direction } = sort;
                    let ordering = match column {
                        Viewers => left.viewer_count.cmp(&right.viewer_count),
                        Uptime => left.uptime.cmp(&right.uptime),
                        // invert this so its a->z not z->a
                        Name => right.user_name.cmp(&left.user_name),
                        _ => unreachable!(),
                    };

                    match direction {
                        Descending => ordering,
                        Ascending => ordering.reverse(),
                    }
                })
                .unwrap_or_else(|| left.viewer_count.cmp(&right.viewer_count))
        });
    }

    #[derive(Default)]
    struct Pad {
        viewers: usize,
        name: usize,
        timestamp: usize,
    }

    let Pad {
        viewers: viewers_max,
        name: name_max,
        timestamp: timestamp_max,
    } = streams
        .values()
        .flatten()
        .fold(Pad::default(), |mut p, stream| {
            p.viewers = p.viewers.max(stream.viewer_count as usize);
            p.name = p.name.max(stream.user_name.len());
            p.timestamp = p.timestamp.max(stream.started_at.len());
            p
        });
    let viewers_max = count_digits(viewers_max as u64);
    let timestamp_max = "uptime".len().max(timestamp_max);

    use std::fmt::Write as _;
    let mut title = String::new();
    for column in &args.visible_columns {
        if !title.is_empty() {
            title.push_str(" | ");
        }

        match column {
            Column::Viewers => {
                write!(
                    &mut title,
                    "{viewers: >viewers_max$}",
                    viewers = " ",
                    viewers_max = viewers_max
                )?;
            }
            Column::Uptime => {
                write!(
                    &mut title,
                    "{uptime: ^timestamp_max$}",
                    uptime = "uptime",
                    timestamp_max = timestamp_max,
                )?;
            }
            Column::Name => {
                write!(
                    &mut title,
                    "{link: ^name_max$}",
                    link = "link",
                    name_max = name_max + "https://twitch.tv/".len()
                )?;
            }
            Column::Title => {
                write!(&mut title, "title")?;
            }
        }
    }
    let line = format!("{:->max$}", "", max = title.len());

    for (n, (category, streams)) in args
        .query
        .iter()
        .filter_map(|q| streams.get(q).map(|s| (q, s)))
        .enumerate()
    {
        for (i, stream) in streams.iter().rev().enumerate() {
            if i == 0 {
                if n > 0 {
                    println!()
                }
                println!("streams for '{category}'", category = category);
                println!("{}", title);
                println!("{}", line);
            }

            let mut output = String::new();
            for column in &args.visible_columns {
                if !output.is_empty() {
                    output.push_str(" | ")
                }

                match column {
                    Column::Viewers => {
                        write!(
                            &mut output,
                            "{viewers: >viewers_max$}",
                            viewers = stream.viewer_count,
                            viewers_max = viewers_max,
                        )?;
                    }
                    Column::Uptime => {
                        write!(
                            &mut output,
                            "{started_at: >timestamp_max$}",
                            started_at = stream.started_at,
                            timestamp_max = timestamp_max,
                        )?;
                    }
                    Column::Name => {
                        write!(
                            &mut output,
                            "https://twitch.tv/{name: <name_max$}",
                            name = stream.user_name,
                            name_max = name_max,
                        )?;
                    }
                    Column::Title => {
                        write!(&mut output, "{title}", title = stream.title)?;
                    }
                }
            }

            println!("{}", output);
        }
    }

    Ok(())
}
