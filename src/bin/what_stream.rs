use std::collections::HashMap;
use std::io::Write;

use what_stream::*;

// TODO other renderers

fn try_enable_colors() {
    if std::env::var("NO_COLOR").is_ok() || (cfg!(windows) && !yansi::Paint::enable_windows_ascii())
    {
        yansi::Paint::disable();
    } else {
        yansi::Paint::enable();
    }
}

fn render_streams<'a, I>(
    out: &mut dyn Write,
    style: &Style,
    theme: &Theme,
    streams: I,
) -> anyhow::Result<()>
where
    I: IntoIterator<Item = (String, &'a [Stream])>,
{
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
            .render(out, style, theme)
        })
}

fn main() -> anyhow::Result<()> {
    let secrets = Secrets::get()?; // TODO maybe do an oauth token flow if we cannot get the secrets

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
