use std::collections::HashMap;

use crate::args::{Column, Direction, Secrets, SortAction};

#[derive(serde::Deserialize, serde::Serialize)]
pub struct Stream {
    pub started_at: String,
    pub title: String,
    pub user_name: String,
    pub user_id: String,
    pub viewer_count: i64,

    #[allow(dead_code)]
    pub language: String,

    #[serde(skip)]
    pub uptime: i64,
}

pub fn fetch_streams<'a>(
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

    let agent = ureq::agent();
    let token = format!("Bearer {}", bearer_oauth);

    let mut cursor = String::new();
    let mut streams = std::iter::from_fn(|| {
        // XXX this is hardcoded (for 'science and technology')
        const SCIENCE_AND_TECH: &str = "509670";

        let resp: Resp<Stream> = agent
            .get("https://api.twitch.tv/helix/streams")
            .query("game_id", SCIENCE_AND_TECH)
            .query("first", "100")
            .query("after", &cursor)
            .set("client-id", client_id)
            .set("authorization", &token)
            .call()
            .ok()?
            .into_json()
            .ok()?;

        match resp.data.is_empty() {
            true => None,
            false => {
                cursor = resp.pagination.cursor;
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
        let duration: chrono::Duration = chrono::Utc::now()
            - stream
                .started_at
                .parse::<chrono::DateTime<chrono::Utc>>()
                .unwrap();

        // TODO do this do differently
        // TODO why?
        let seconds = duration.num_seconds();
        let hours = (seconds / 60) / 60;
        let minutes = (seconds / 60) % 60;

        let started = if hours > 0 {
            format!(
                "{hours} hour{h_plural} {minutes} minute{m_plural}",
                hours = hours,
                minutes = minutes,
                h_plural = if hours > 1 { "s" } else { "" },
                m_plural = if minutes > 1 { "s" } else { "" },
            )
        } else {
            format!(
                "{minutes} minute{m_plural}",
                minutes = minutes,
                m_plural = if minutes > 1 { "s" } else { "" }
            )
        };

        stream.uptime = seconds;
        stream.started_at = started;
    });

    fn get_usernames<'b: 'a, 'a, I>(
        agent: &ureq::Agent,
        ids: I,
        client_id: &str,
        token: &str,
    ) -> anyhow::Result<HashMap<String, String>>
    where
        I: Iterator<Item = &'b String> + 'a,
    {
        #[derive(serde::Deserialize)]
        struct Resp<T> {
            data: Vec<T>,
        }

        #[derive(serde::Deserialize)]
        struct User {
            id: String,
            login: String,
        }

        let mut req = agent.get("https://api.twitch.tv/helix/users");
        for (k, v) in std::iter::repeat("id").zip(ids) {
            req = req.query(k, v);
        }
        req = req.set("client-id", client_id).set("authorization", &token);

        let resp: Resp<User> = req.call()?.into_json()?;
        Ok(resp.data.into_iter().map(|u| (u.id, u.login)).collect())
    }

    for streams in streams.chunks_mut(100) {
        for (k, v) in get_usernames(
            &agent,
            streams.iter_mut().map(|(_, u)| &u.user_id),
            &client_id,
            &token,
        )? {
            if let Some((_, stream)) = streams.iter_mut().find(|(_, s)| s.user_id == k) {
                stream.user_name = v;
            }
        }
    }

    Ok(streams)
}

pub fn sort_streams(streams: &mut Vec<Stream>, option: Option<SortAction>) {
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
                };

                match direction {
                    Ascending => ordering,
                    Descending => ordering.reverse(),
                }
            })
            .unwrap_or_else(|| left.viewer_count.cmp(&right.viewer_count))
    });

    // sometimes the api hiccups -- this'll ensure we'll just get uniques
    streams.dedup_by(|a, b| a.user_name == b.user_name);
}

fn trim_word_boundaries(s: &str) -> &str {
    const HEAD: &str = "([{";
    const TAIL: &str = ",.!?-:}])";
    s.trim_start_matches(|c| HEAD.contains(c))
        .trim_end_matches(|c| TAIL.contains(c))
        .trim()
}
