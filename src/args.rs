use anyhow::Context as _;

use crate::render::Style;

#[derive(Debug)]
pub struct Args {
    pub sort: Option<SortAction>,
    pub query: Vec<String>,
    pub json: bool,
    pub style: Style,
}

impl Args {
    pub fn parse() -> anyhow::Result<Self> {
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

        let query = args
            .finish()
            .into_iter()
            .map(|s| s.to_string_lossy().to_string())
            .collect();
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
        println!("{}", include_str!("../assets/short_help.txt"));
    }

    fn print_long_help() {
        Self::print_short_help();
        println!("{}", include_str!("../assets/long_help.txt"));
    }

    fn print_version() {
        println!("{} {}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));
    }
}

pub struct Secrets {
    pub client_id: String,
    pub bearer_oauth: String,
}

impl Secrets {
    pub fn get() -> anyhow::Result<Self> {
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

#[derive(Debug, Copy, Clone)]
pub struct SortAction {
    pub column: Column,
    pub direction: Direction,
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
pub enum Column {
    Viewers,
    Uptime,
    Name,
}

#[derive(Debug, Copy, Clone, PartialOrd, PartialEq)]
pub enum Direction {
    Descending,
    Ascending,
}
