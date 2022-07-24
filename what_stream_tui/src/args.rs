use anyhow::Context as _;

use what_stream_core::SortAction;

#[derive(Debug)]
pub struct Args {
    pub sort: Option<SortAction>,
    pub query: Vec<String>,
    pub languages: Vec<String>,
    pub json: bool,
    pub demo: bool,
}

// TODO reject unknown arguments
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

        if args.contains("--print-cache-dir") {
            println!(
                "{}",
                crate::Config::get_cache_dir()
                    .with_context(|| "your system does not have a cache directory")?
                    .to_string_lossy()
            );
            std::process::exit(0)
        }

        let demo = args.contains("--demo");

        let json = args.contains(["-j", "--json"]);
        let sort = args.opt_value_from_str(["-s", "--sort"])?;

        let languages: Vec<String> = args.values_from_str(["-l", "--language"])?;

        type PicoResult<T> = Result<T, pico_args::Error>;
        let query: Vec<_> = std::iter::from_fn(|| {
            match args.free_from_fn(|s| {
                if s.starts_with('-') {
                    return Err("unknown flag");
                }
                Ok(s.to_string())
            }) {
                Err(pico_args::Error::MissingArgument) => None,
                other => Some(other),
            }
        })
        .collect::<PicoResult<_>>()?;

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
