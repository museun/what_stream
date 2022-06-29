use anyhow::Context as _;

use crate::{WHAT_STREAM_CLIENT_ID, WHAT_STREAM_CLIENT_SECRET};

#[derive(Debug)]
pub struct Args {
    pub sort: Option<SortAction>,
    pub query: Vec<String>,
    pub languages: Vec<String>,
    pub json: bool,
    pub demo: bool,
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

        if args.contains("--print-default-config") {
            println!("{}", crate::Config::default_formatted_toml());
            std::process::exit(0)
        }

        if args.contains("--print-config-path") {
            println!(
                "{}",
                crate::Config::get_config_path()
                    .with_context(|| "your system does not have a configuration directory")?
                    .to_string_lossy()
            );
            std::process::exit(0)
        }

        let demo = args.contains("--demo");

        let json = args.contains(["-j", "--json"]);
        let sort = args.opt_value_from_str(["-s", "--sort"])?;

        let languages: Vec<String> = args.values_from_str(["-l", "--language"])?;

        let query = args
            .finish()
            .into_iter()
            .map(|s| s.to_string_lossy().to_string())
            .collect::<Vec<_>>();

        Ok(Self {
            sort,
            query,
            languages,
            json,
            demo,
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
            name => anyhow::bail!(
                "invalid column: {}. supported columns: [viewers | uptime | name]",
                name
            ),
        };

        let direction = iter
            .next()
            .map(|tail| match tail {
                "asc" | "ascending" => Ok(Direction::Ascending),
                "desc" | "descending" | "" => Ok(Direction::Descending),
                dir => anyhow::bail!("invalid direction: {}. supported directions: [asc | ascending | desc | descending]", dir),
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

#[derive(Debug, Copy, Clone, PartialOrd, PartialEq, Eq)]
pub enum Direction {
    Descending,
    Ascending,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct AppAccess {
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub expires_in: u64,
    pub token_type: String,
}

impl AppAccess {
    pub fn get() -> anyhow::Result<Self> {
        ureq::post("https://id.twitch.tv/oauth2/token")
            .query("client_id", WHAT_STREAM_CLIENT_ID)
            .query("client_secret", WHAT_STREAM_CLIENT_SECRET)
            .query("grant_type", "client_credentials")
            .call()?
            .into_json()
            .map_err(Into::into)
    }
}
