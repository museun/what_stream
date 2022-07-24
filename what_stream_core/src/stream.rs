use std::collections::HashMap;

#[derive(Debug, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
pub struct Stream {
    pub started_at: Box<str>,
    pub title: Box<str>,
    pub user_name: Box<str>,
    pub user_id: Box<str>,
    pub viewer_count: i64,
    pub language: Box<str>,

    #[serde(default)]
    pub tag_ids: Option<Box<[Box<str>]>>,

    #[serde(skip_deserializing)]
    pub user_tag_map: HashMap<Box<str>, Box<str>>,

    #[serde(skip_deserializing)]
    pub uptime: i64,
}

impl Ord for Stream {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.user_id.cmp(&other.user_id)
    }
}

impl PartialOrd for Stream {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.user_id.partial_cmp(&other.user_id)
    }
}
