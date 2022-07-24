use crate::AppAccess;

pub struct StatefulAgent {
    access: AppAccess,
    agent: ureq::Agent,
}

impl StatefulAgent {
    pub fn new(access: AppAccess) -> Self {
        Self {
            access,
            agent: ureq::Agent::new(),
        }
    }

    pub fn get_query<'a, T, Q>(&self, ep: &str, query: Q) -> anyhow::Result<T>
    where
        T: serde::de::DeserializeOwned,
        Q: IntoIterator<Item = (&'a str, &'a str)>,
    {
        query
            .into_iter()
            .fold(self.agent.get(ep), |req, (k, v)| req.query(k, v))
            .set("client-id", self.access.get_client_id())
            .set("authorization", self.access.get_bearer_token())
            .call()?
            .into_json::<T>()
            .map_err(Into::into)
    }
}
