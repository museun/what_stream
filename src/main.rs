fn main() -> anyhow::Result<()> {
    #[derive(Debug, serde::Deserialize)]
    struct Resp<T> {
        data: Vec<T>,
        pagination: Pagination,
    }
    #[derive(Default, Debug, serde::Deserialize)]
    struct Pagination {
        #[serde(default)]
        cursor: String,
    }
    #[derive(Debug, serde::Deserialize)]
    struct Stream {
        id: String,
        language: String,
        started_at: String,
        thumbnail_url: String,
        title: String,
        user_id: String,
        user_name: String,
        viewer_count: i64,
    }

    let client_id = std::env::var("WHAT_STREAM_CLIENT_ID").unwrap_or_else(|_| {
        eprintln!("please set 'WHAT_STREAM_CLIENT_ID' to your Twitch Client ID");
        std::process::exit(1)
    });
    let bearer_oauth = std::env::var("WHAT_STREAM_BEARER_OAUTH").unwrap_or_else(|_| {
        eprintln!("please set 'WHAT_STREAM_BEARER_OAUTH' to your Twitch Bearer OAuth token");
        std::process::exit(1)
    });

    let query = std::env::args().skip(1).collect::<String>();
    let query = query.trim();
    if query.is_empty() {
        eprintln!("provide a title to search for");
        std::process::exit(1)
    }

    fn remove_word_boundaries(s: &str) -> &str {
        const PAT: &str = ",.!?-";
        s.trim_end_matches(PAT).trim()
    }

    let query = query
        .split(',')
        .map(remove_word_boundaries)
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>();

    let mut after = String::new();
    let mut streams = std::iter::from_fn(|| {
        let resp: Resp<Stream> = attohttpc::get("https://api.twitch.tv/helix/streams")
            .param("game_id", "509670") // TODO this is hardcoded (for 'science and technology')
            .param("first", "100")
            .param("after", &after)
            .header("client-id", &client_id)
            .bearer_auth(&bearer_oauth)
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
            .filter(|s| !s.is_empty())
            .map(|s| s.to_lowercase())
        {
            for q in &query {
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

        stream.started_at = started;
    });

    streams.sort_unstable_by_key(|(_, c)| c.viewer_count);

    fn count_digits(d: u64) -> usize {
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

    let max_viewers = streams
        .iter()
        .map(|(_, c)| c.viewer_count)
        .max()
        .unwrap_or(1);
    let max_viewers = count_digits(max_viewers as u64);
    let max_name = streams
        .iter()
        .map(|(_, c)| c.user_name.len())
        .max()
        .unwrap_or(1);

    let max_timestamp = streams
        .iter()
        .map(|(_, c)| c.started_at.len())
        .max()
        .unwrap_or(1)
        .max("uptime".len());

    let map = streams.iter().fold(
        std::collections::BTreeMap::<_, Vec<_>>::new(),
        |mut m, (a, c)| {
            m.entry(a).or_default().push(c);
            m
        },
    );

    let title = format!(
        "{viewers: >max_viewers$} | {uptime: ^max_timestamp$} | {link: ^max_name$} | title",
        viewers = " ",
        uptime = "uptime",
        link = "link",
        max_viewers = max_viewers,
        max_timestamp = max_timestamp,
        max_name = max_name + "https://twitch.tv/".len(),
    );

    let line = format!("{:->max$}", "", max = title.len());

    for (n, (category, streams)) in map.iter().enumerate() {
        for (i, stream) in streams.iter().rev().enumerate() {
            if i == 0 {
                if n > 0 {
                    println!()
                }
                println!("streams for '{category}'", category = category,);
                println!("{}", title);
                println!("{}", line);
            }

            println!(
                "{viewers: >max_viewers$} | {started_at: >max_timestamp$} | https://twitch.tv/{name: <max_name$} | {title}",
                viewers = stream.viewer_count,
                started_at = stream.started_at,
                title = stream.title,
                name = stream.user_name,
                max_viewers = max_viewers,
                max_timestamp = max_timestamp,
                max_name = max_name,
            );
        }
    }

    Ok(())
}
