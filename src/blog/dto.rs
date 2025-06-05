use serde::{ Deserialize, Serialize };

use crate::blog::model::Post;

#[derive(Debug, Deserialize)]
pub struct CreatePost {
    pub title: String,
    pub description: String,
    pub body: String,
    #[serde(default)]
    pub tags: Vec<String>,
    pub thumbnail: String,
    pub thumbnail_blur: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)] 
pub struct PostListResponse {
    pub total_count: i64,
    pub posts: Vec<Post>,
}

#[derive(Debug, Deserialize)]
pub struct UpdatePost {
    pub title: Option<String>,
    pub body: Option<String>,
    pub description: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct BlurRequest {
    pub url: String,
}

#[derive(Debug, Serialize)]
pub struct BlurResponse {
    pub data_url: String,
}
