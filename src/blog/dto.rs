use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct CreatePost {
    pub title: String,
    pub body: String,
    #[serde(default)]
    pub tags: Vec<String>,
    pub thumbnail: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdatePost {
    pub title: Option<String>,
    pub body: Option<String>,
}
