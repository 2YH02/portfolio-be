use serde::Serialize;

#[derive(Serialize)]
pub struct Likes {
    pub like_count: i64,
    pub liked: bool,
}
