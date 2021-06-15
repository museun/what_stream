use std::collections::HashMap;

use crate::{
    args::{AppAccess, Column, Direction, SortAction},
    SCIENCE_AND_TECH_CATEGORY, WHAT_STREAM_CLIENT_ID,
};

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct Stream {
    pub started_at: String,
    pub title: String,
    pub user_name: String,
    pub user_id: String,
    pub viewer_count: i64,
    pub language: String,

    #[serde(skip)]
    pub uptime: i64,
}

pub fn fetch_streams<'a>(
    query: &'a [String],
    languages: &[String],
    app_access: &AppAccess,
) -> anyhow::Result<Vec<(&'a String, Stream)>> {
    let agent = ureq::agent();
    let token = format!("Bearer {}", app_access.access_token);

    let mut streams = iterater_and_filter(&agent, query, languages, &token);

    // fix up the time
    for (_, stream) in &mut streams {
        let (seconds, started_at) = format_time(&stream.started_at);
        stream.uptime = seconds;
        stream.started_at = started_at;
    }

    // then fetch usernames for each userid
    for streams in streams.chunks_mut(100) {
        let user_ids = streams.iter_mut().map(|(_, u)| &u.user_id);
        for (k, v) in get_usernames(&agent, user_ids, &token)? {
            // this is sorta quadratic
            if let Some((_, stream)) = streams.iter_mut().find(|(_, s)| s.user_id == k) {
                stream.user_name = v;
            }
        }
    }

    Ok(streams)
}

pub fn sort_streams(streams: &mut Vec<Stream>, option: Option<SortAction>) {
    use {Column::*, Direction::*};

    // sometimes the api hiccups -- this'll ensure we'll just get uniques
    streams.sort_unstable_by(|a, b| a.user_id.cmp(&b.user_id));
    streams.dedup_by(|a, b| a.user_id == b.user_id);

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
}

fn fetch_streams_inner(
    agent: &ureq::Agent,
    token: &str,
    cursor: &mut String,
) -> Option<Vec<Stream>> {
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

    let resp: Resp<Stream> = agent
        .get("https://api.twitch.tv/helix/streams")
        .query("game_id", SCIENCE_AND_TECH_CATEGORY)
        .query("first", "100")
        .query("after", cursor)
        .set("client-id", WHAT_STREAM_CLIENT_ID)
        .set("authorization", token)
        .call()
        .ok()?
        .into_json()
        .ok()?;

    if !resp.data.is_empty() {
        *cursor = resp.pagination.cursor;
        return Some(resp.data);
    }
    None
}

fn iterater_and_filter<'a>(
    agent: &ureq::Agent,
    query: &'a [String],
    languages: &[String],
    token: &str,
) -> Vec<(&'a String, Stream)> {
    let mut cursor = String::new();
    std::iter::from_fn(|| fetch_streams_inner(agent, token, &mut cursor))
        .flatten()
        .filter(|stream| {
            languages.is_empty()
                || languages
                    .iter()
                    .any(|s| s.eq_ignore_ascii_case(&stream.language))
        })
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
        .collect()
}

fn get_usernames<'b: 'a, 'a, I>(
    agent: &ureq::Agent,
    ids: I,
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

    std::iter::repeat("id")
        .zip(ids)
        .fold(
            agent.get("https://api.twitch.tv/helix/users"),
            |req, (k, v)| req.query(k, v),
        )
        .set("client-id", WHAT_STREAM_CLIENT_ID)
        .set("authorization", token)
        .call()?
        .into_json::<Resp<User>>()?
        .data
        .into_iter()
        .map(|u| (u.id, u.login))
        .map(Ok)
        .collect()
}

fn format_time(started_at: &str) -> (i64, String) {
    use chrono::*;
    let duration: Duration = Utc::now()
        - started_at
            .parse::<DateTime<Utc>>()
            .expect("valid timestamp");

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

    (seconds, started)
}

fn trim_word_boundaries(s: &str) -> &str {
    const HEAD: &str = "([{";
    const TAIL: &str = ",.!?-:}])";
    s.trim_start_matches(|c| HEAD.contains(c))
        .trim_end_matches(|c| TAIL.contains(c))
        .trim()
}
