use std::collections::{HashMap, HashSet};

use crate::{
    args::{AppAccess, Column, Direction, SortAction},
    config::TagCache,
    SCIENCE_AND_TECH_CATEGORY, SOFTWARE_AND_GAME_DEV_CATEGORY, WHAT_STREAM_CLIENT_ID,
};

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct Stream {
    pub started_at: Box<str>,
    pub title: Box<str>,
    pub user_name: Box<str>,
    pub user_id: Box<str>,
    pub viewer_count: i64,
    pub language: Box<str>,

    pub tag_ids: Box<[Box<str>]>,

    #[serde(skip_deserializing)]
    pub user_tag_map: HashMap<Box<str>, Box<str>>,

    #[serde(skip_deserializing)]
    pub uptime: i64,
}

pub fn fetch_streams<'a>(
    query: &'a [String],
    languages: &[String],
    app_access: &AppAccess,
    tag_cache: &mut TagCache,
) -> anyhow::Result<Vec<(&'a String, Stream)>> {
    let agent = ureq::agent();
    let token = format!("Bearer {}", app_access.access_token);

    let mut streams = get_streams(&agent, query, languages, tag_cache, &token);

    // fix up the time
    for (_, stream) in &mut streams {
        let (seconds, started_at) = format_time(&stream.started_at);
        stream.uptime = seconds;
        stream.started_at = started_at.into();
    }

    // then fetch usernames for each userid
    for streams in streams.chunks_mut(100) {
        let user_ids = streams.iter_mut().map(|(_, u)| &*u.user_id);
        for (k, v) in get_usernames(&agent, user_ids, &token)? {
            if let Some((_, stream)) = streams.iter_mut().find(|(_, s)| *s.user_id == k) {
                stream.user_name = v.into();
            }
        }
    }

    for (_, stream) in &mut streams {
        for id in &*stream.tag_ids {
            if let Some(tag) = tag_cache.cache.get(id) {
                stream.user_tag_map.insert(id.clone(), tag.clone());
            }
        }
    }

    Ok(streams)
}

pub fn sort_streams(streams: &mut Vec<Stream>, option: Option<SortAction>) {
    use {Column::*, Direction::*};

    // TODO figure out a way around this: https://github.com/twitchdev/issues/issues/18
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

fn lookup_ids<'a>(
    agent: &ureq::Agent,
    token: &str,
    ids: impl IntoIterator<Item = &'a str> + 'a,
    memo: &mut TagCache,
) {
    #[derive(serde::Deserialize)]
    struct Tag {
        tag_id: Box<str>,
        #[serde(default)]
        is_auto: bool,
        localization_names: HashMap<Box<str>, Box<str>>,
    }

    type Tags = data::Resp<Tag>;

    let resp = match std::iter::repeat("tag_id")
        .zip(ids)
        .fold(
            agent.get("https://api.twitch.tv/helix/tags/streams"),
            |req, (k, v)| req.query(k, v),
        )
        .set("client-id", WHAT_STREAM_CLIENT_ID)
        .set("authorization", token)
        .call()
    {
        Ok(resp) => resp.into_reader(),
        Err(..) => return, // TODO report this
    };

    let tags = match serde_json::from_reader::<_, Tags>(resp) {
        Ok(tags) => tags,
        Err(..) => return, // TODO report this
    };

    for data in tags.data.into_iter().filter(|s| !s.is_auto) {
        if let Some(name) = { data.localization_names }.remove("en-us") {
            memo.cache.insert(data.tag_id, name);
        }
    }
}

mod data {
    #[derive(serde::Deserialize)]
    pub struct Resp<T> {
        pub data: Vec<T>,
        pub pagination: Pagination,
    }

    #[derive(Default, serde::Deserialize)]
    pub struct Pagination {
        #[serde(default)]
        pub cursor: String,
    }
}

fn get_streams<'a>(
    agent: &ureq::Agent,
    query: &'a [String],
    languages: &[String],
    tags: &mut TagCache,
    token: &str,
) -> Vec<(&'a String, Stream)> {
    type Streams = data::Resp<Stream>;

    let mut streams = Vec::new();
    let mut cursor = String::new();
    while let Ok(resp) = agent
        .get("https://api.twitch.tv/helix/streams")
        .query("game_id", SCIENCE_AND_TECH_CATEGORY)
        .query("game_id", SOFTWARE_AND_GAME_DEV_CATEGORY)
        .query("first", "100")
        .query("after", &cursor)
        .set("client-id", WHAT_STREAM_CLIENT_ID)
        .set("authorization", token)
        .call()
        .map(ureq::Response::into_reader)
    {
        let mut resp = match serde_json::from_reader::<_, Streams>(resp) {
            Err(..) => break, // TODO report this
            Ok(resp) if resp.data.is_empty() => break,
            Ok(resp) if resp.pagination.cursor == cursor => break,
            Ok(resp) => resp,
        };

        cursor = resp.pagination.cursor;
        let mut temp = std::mem::take(&mut resp.data);
        if !languages.is_empty() {
            temp.retain(|stream| {
                languages
                    .iter()
                    .any(|lang| stream.language.eq_ignore_ascii_case(lang))
            });
        }

        let unknown_ids: HashSet<&str> = temp
            .iter()
            .flat_map(|s| &*s.tag_ids)
            .filter_map(|s| (!tags.cache.contains_key(&**s)).then(|| &**s))
            .collect();

        lookup_ids(agent, token, unknown_ids, tags);

        'stream: for stream in temp {
            for id in &*stream.tag_ids {
                if let Some(tag) = tags.cache.get(id) {
                    for q in query {
                        if q.eq_ignore_ascii_case(tag) {
                            streams.push((q, stream));
                            continue 'stream;
                        }
                    }
                }
            }

            for part in stream
                .title
                .split(' ')
                .map(trim_word_boundaries)
                .filter(|s| !s.is_empty())
            {
                for q in query {
                    if q.eq_ignore_ascii_case(part) {
                        streams.push((q, stream));
                        continue 'stream;
                    }
                }
            }
        }
    }

    streams
}

fn get_usernames<'b: 'a, 'a, I>(
    agent: &ureq::Agent,
    ids: I,
    token: &str,
) -> anyhow::Result<HashMap<String, String>>
where
    I: Iterator<Item = &'b str> + 'a,
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
