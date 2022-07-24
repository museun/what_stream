use std::collections::HashMap;

#[derive(Debug, serde::Deserialize, serde::Serialize)]
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
