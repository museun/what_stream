#[derive(serde::Deserialize)]
pub struct Resp<T> {
    pub data: Vec<T>,
    #[serde(default)]
    pub pagination: Pagination,
}

#[derive(Default, serde::Deserialize)]
pub struct Pagination {
    #[serde(default)]
    pub cursor: String,
}
