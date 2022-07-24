use crate::app::App;

pub enum Action {
    Remove(String),
    Refresh(String),
}

pub enum Lookup<'a> {
    Refresh(&'a mut App),
    New(&'a mut App),
}

impl<'a> Lookup<'a> {
    pub fn lookup<'b>(
        &mut self,
        query: impl Iterator<Item = &'b str> + Clone,
    ) -> anyhow::Result<()> {
        let is_new = matches!(self, Self::New(..));

        let app = match self {
            Self::Refresh(app) | Self::New(app) => app,
        };

        let id = app.query_map.len();
        app.query_map.extend(
            query
                .clone()
                .enumerate()
                .map(|(k, v)| (v.to_string(), (k + id))),
        );

        let query = query.collect::<Vec<_>>();

        let mut results = match app.what_stream.fetch_streams(&query) {
            Ok(results) if results.is_empty() => {
                anyhow::bail!(
                    "nothing was found for:\n{}",
                    query.iter().fold(String::new(), |mut a, c| {
                        if !a.is_empty() {
                            a.push('\n');
                        }
                        a.push_str(c);
                        a
                    })
                );
            }
            Ok(results) => results,
            err @ Err(..) => return err.map(drop).map_err(Into::into),
        };

        results.sort_unstable_by(|&(l, ..), &(r, ..)| {
            let get = |i| app.query_map.get(i).copied().unwrap_or(usize::MAX);
            get(l).cmp(&get(r))
        });

        use indexmap::map::Entry;
        for (key, value) in results {
            match app.results.entry(key.to_string()) {
                Entry::Occupied(mut v) if !is_new => {
                    v.get_mut().replace(value);
                }
                Entry::Occupied(mut v) => {
                    v.get_mut().insert(value);
                }
                Entry::Vacant(v) => {
                    v.insert([value].into_iter().collect());
                }
            }
        }

        Ok(())
    }
}
