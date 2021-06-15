use std::collections::HashMap;
use std::io::Write;

use what_stream::*;

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
        .try_for_each(|(n, (category, streams))| {
            if n > 0 {
                writeln!(out)?;
            }
            Entries {
                category: &category,
                streams,
            }
            .render(out, glyphs, colors)
        })
}

fn append_maybe(left: &mut Vec<String>, right: &[String]) {
    if left.is_empty() {
        left.extend(right.iter().cloned());
    }
    left.retain(|s| !s.is_empty());
}

fn show_demo(config: &Config) -> anyhow::Result<()> {
    try_enable_colors();
    let out = std::io::stdout();
    let mut out = out.lock();

    let Appearance { glyphs, colors } = &config.appearance;
    what_stream::Demo::show_off(&mut out, glyphs, colors)?;
    Ok(())
}

fn main() -> anyhow::Result<()> {
    let mut args = Args::parse()?;
    let config: Config = Config::get_config_path()
        .and_then(|f| std::fs::read(f).ok())
        .map(|d| toml::from_slice(&d).with_context(|| "invalid toml"))
        .transpose()?
        .unwrap_or_default();

    append_maybe(&mut args.languages, &*config.parameters.languages);
    append_maybe(&mut args.query, &*config.parameters.query);

    if args.demo {
        show_demo(&config)?;
        std::process::exit(0);
    }

    if args.query.is_empty() {
        eprintln!("please provide something to filter by");
        std::process::exit(1)
    }

    // TODO read from the config to see if we should override the token?
    let app_access = AppAccess::get()?;

    let mut streams: HashMap<_, Vec<_>> = fetch_streams(&args.query, &args.languages, &app_access)?
        .into_iter()
        .fold(Default::default(), |mut map, (category, stream)| {
            map.entry(category.clone()).or_default().push(stream);
            map
        });

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

    render_streams(&mut out, &config, streams)
}
