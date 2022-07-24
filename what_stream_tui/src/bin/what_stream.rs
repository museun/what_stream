use std::collections::HashMap;
use std::io::Write;

use what_stream_core::*;
use what_stream_tui::*;

use anyhow::Context as _;

// TODO other renderers

fn try_enable_colors() {
    if std::env::var("NO_COLOR").is_ok() || (cfg!(windows) && !yansi::Paint::enable_windows_ascii())
    {
        yansi::Paint::disable();
    } else {
        yansi::Paint::enable();
    }
}

fn render_streams<'a, I>(out: &mut dyn Write, config: &Config, streams: I) -> anyhow::Result<()>
where
    I: IntoIterator<Item = (String, &'a [Stream])>,
{
    let Appearance { glyphs, colors } = &config.appearance;

    streams
        .into_iter()
        .enumerate()
        .try_for_each(|(n, (query, streams))| {
            if n > 0 {
                writeln!(out)?;
            }
            Entries {
                query: &query,
                streams,
            }
            .render(out, glyphs, colors)
        })
}

fn append_maybe<T: Clone>(left: &mut Vec<T>, right: &[T], retain: fn(&T) -> bool) {
    if left.is_empty() {
        left.extend(right.iter().cloned());
    }
    left.retain(retain);
}

fn show_demo(config: &Config) -> anyhow::Result<()> {
    try_enable_colors();
    let Appearance { glyphs, colors } = &config.appearance;
    what_stream_tui::Demo::show_off(&mut std::io::stdout().lock(), glyphs, colors)
}

fn main() -> anyhow::Result<()> {
    alto_logger::init_alt_term_logger()?;

    let mut args = match Args::parse() {
        Ok(args) => args,
        Err(err) => {
            eprintln!(
                "{err}\ntry running: {program_name} --help",
                err = err,
                program_name = env!("CARGO_CRATE_NAME")
            );
            std::process::exit(1)
        }
    };

    let config_path = Config::get_config_path().with_context(|| "cannot get configuration path")?;
    let config: Config = match std::fs::read(&config_path)
        .ok()
        .map(|d| toml::from_slice(&d).with_context(|| "invalid toml"))
        .transpose()?
    {
        Some(config) => config,
        None => {
            let cache_dir = Config::get_cache_dir().with_context(|| "cannot get cache path")?;
            eprintln!(
                r#"
a configuration file was not found at:
{config_file}

this will create the directory if it does not exist
this will also create a cache directory at: {cache_dir}
and then create the configuration file

you should edit it -- the only required values are in the 'auth' section
"#,
                config_file = config_path.display(),
                cache_dir = cache_dir.display()
            );

            Config::make_required_dirs()?;
            return Config::make_default_config();
        }
    };

    // TODO this is ugly
    append_maybe(&mut args.languages, &config.parameters.languages, |s| {
        !s.is_empty()
    });
    append_maybe(&mut args.query, &config.parameters.query, |s| !s.is_empty());

    if args.demo {
        show_demo(&config)?;
        std::process::exit(0);
    }

    if args.query.is_empty() {
        eprintln!("please provide something to filter by");
        std::process::exit(1)
    }

    let app_access = AppAccess::create(&config.auth.client_id, &config.auth.client_secret)?;

    let tag_cache_path = Config::get_cache_dir()
        .with_context(|| "cannot get the cache directory")?
        .join("tags_cache.json");

    let mut what_stream = WhatStream::new(
        &args.query,
        &args.languages,
        &[SCIENCE_AND_TECH_CATEGORY, SOFTWARE_AND_GAME_DEV_CATEGORY],
        app_access,
        &tag_cache_path,
    );

    log::trace!("starting fetch");
    let mut streams: HashMap<_, Vec<_>> = what_stream.fetch_streams()?.into_iter().fold(
        Default::default(),
        |mut map, (category, stream)| {
            map.entry(category).or_default().push(stream);
            map
        },
    );

    if args.json {
        println!("{}", serde_json::to_string_pretty(&streams)?);
        std::process::exit(0)
    }

    for streams in streams.values_mut() {
        WhatStream::sort_streams(streams, args.sort)
    }

    try_enable_colors();

    let streams = args
        .query
        .into_iter()
        .filter_map(|q| streams.get(&*q).map(|s| (q, s.as_slice())));

    render_streams(&mut std::io::stdout().lock(), &config, streams)
}
