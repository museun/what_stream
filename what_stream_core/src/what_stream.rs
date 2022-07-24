use std::{
    collections::{HashMap, HashSet},
    path::Path,
};

use crate::{http::StatefulAgent, AppAccess, Column, Direction, SortAction, Stream, TagCache};

pub struct WhatStream {
    query: Vec<String>,
    languages: Vec<String>,
    tag_cache: TagCache,
    categories: Vec<String>,
    agent: crate::http::StatefulAgent,
}

impl WhatStream {
    pub fn new<Q, L, C>(
        query: &[Q],
        languages: &[L],
        categories: &[C],
        app_access: AppAccess,
        tag_cache_path: &Path,
    ) -> Self
    where
        Q: ToString,
        L: ToString,
        C: ToString,
    {
        Self {
            query: query.iter().map(ToString::to_string).collect(),
            languages: languages.iter().map(ToString::to_string).collect(),
            categories: categories.iter().map(ToString::to_string).collect(),
            agent: StatefulAgent::new(app_access),
            tag_cache: TagCache::load_cache(tag_cache_path),
        }
    }

    // TODO this should return an iterator so the UI won't block
    pub fn fetch_streams(&mut self) -> anyhow::Result<Vec<(String, Stream)>> {
        let mut streams = self.get_streams();

        // fix up the time
        for (_, stream) in &mut streams {
            let (seconds, started_at) = crate::util::format_time(&stream.started_at);
            stream.uptime = seconds;
            stream.started_at = started_at.into();
        }

        // then fetch usernames for each userid
        for streams in streams.chunks_mut(100) {
            let user_ids = streams.iter_mut().map(|(_, u)| &*u.user_id);
            for (k, v) in self.get_usernames(user_ids)? {
                if let Some((_, stream)) = streams.iter_mut().find(|(_, s)| *s.user_id == k) {
                    stream.user_name = v.into();
                }
            }
        }

        for (_, stream) in &mut streams {
            for id in stream.tag_ids.iter().flat_map(|s| &**s) {
                if let Some(tag) = self.tag_cache.cache.get(id) {
                    stream.user_tag_map.insert(id.clone(), tag.clone());
                }
            }
        }

        self.tag_cache.sync()?;
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

    fn lookup_tag_ids<'a>(&mut self, ids: impl IntoIterator<Item = &'a str> + 'a) {
        #[derive(serde::Deserialize)]
        struct Tag {
            tag_id: Box<str>,
            #[serde(default)]
            is_auto: bool,
            localization_names: HashMap<Box<str>, Box<str>>,
        }

        type Tags = crate::data::Resp<Tag>;

        let tags = match self.agent.get_query::<Tags, _>(
            "https://api.twitch.tv/helix/tags/streams",
            std::iter::repeat("tag_id").zip(ids),
        ) {
            Ok(tags) => tags,
            Err(err) => {
                log::error!("cannot deserialize tags: {}", err);
                return;
            }
        };

        for data in tags.data.into_iter().filter(|s| !s.is_auto) {
            if let Some(name) = { data.localization_names }.remove("en-us") {
                self.tag_cache.cache.insert(data.tag_id, name);
            }
        }
    }

    fn get_streams(&mut self) -> Vec<(String, Stream)> {
        type Streams = crate::data::Resp<Stream>;

        log::trace!("tag cache: {}", self.tag_cache.cache.len());

        let mut streams = Vec::new();
        let mut cursor = String::new();
        while let Ok(resp) = self.agent.get_query::<Streams, _>(
            "https://api.twitch.tv/helix/streams",
            [("first", "100"), ("after", &cursor)]
                .into_iter()
                .chain(std::iter::repeat("game_id").zip(self.categories.iter().map(|s| &**s))),
        ) {
            let mut resp = match resp {
                resp if resp.data.is_empty() => break,
                resp if resp.pagination.cursor == cursor => break,
                resp => resp,
            };

            cursor = resp.pagination.cursor;
            let mut temp = std::mem::take(&mut resp.data);
            if !self.languages.is_empty() {
                log::debug!("got {} new streams", temp.len());
                temp.retain(|stream| {
                    self.languages
                        .iter()
                        .any(|lang| stream.language.eq_ignore_ascii_case(lang))
                });
                log::debug!(
                    "filtered to {} streams based on language {:?}",
                    temp.len(),
                    self.languages
                );
            }

            let unknown_ids: HashSet<&str> = temp
                .iter()
                .flat_map(|s| s.tag_ids.iter().flat_map(|s| &**s))
                .filter_map(|s| (!self.tag_cache.cache.contains_key(&**s)).then_some(&**s))
                .collect();

            log::debug!("new unknown ids: {}", unknown_ids.len());

            self.lookup_tag_ids(unknown_ids);

            let old = streams.len();
            'stream: for stream in temp {
                for id in stream.tag_ids.iter().flat_map(|s| &**s) {
                    if let Some(tag) = self.tag_cache.cache.get(id) {
                        for q in &self.query {
                            if q.eq_ignore_ascii_case(tag) {
                                streams.push((q.clone(), stream));
                                continue 'stream;
                            }
                        }
                    }
                }

                for part in stream
                    .title
                    .split(' ')
                    .map(crate::util::trim_word_boundaries)
                    .filter(|s| !s.is_empty())
                {
                    for q in &self.query {
                        if q.eq_ignore_ascii_case(part) {
                            streams.push((q.clone(), stream));
                            continue 'stream;
                        }
                    }
                }
            }

            let new = streams.len().saturating_sub(old);
            if new == 0 {
                break;
            }
            log::debug!("new streams: {}", new);
        }

        log::debug!("total streams: {}", streams.len());
        streams
    }

    fn get_usernames<'a>(
        &self,
        ids: impl Iterator<Item = &'a str>,
    ) -> anyhow::Result<HashMap<String, String>> {
        #[derive(serde::Deserialize)]
        struct User {
            id: String,
            login: String,
        }

        let resp: crate::data::Resp<User> = self.agent.get_query(
            "https://api.twitch.tv/helix/users",
            std::iter::repeat("id").zip(ids),
        )?;

        resp.data
            .into_iter()
            .map(|u| (u.id, u.login))
            .map(Ok)
            .collect()
    }
}
