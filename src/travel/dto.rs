use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct TravelLikeResponse {
    pub like_count: i32,
}

#[derive(Debug, Deserialize)]
pub struct CreateTravelComment {
    pub author: String,
    pub password: String,
    pub content: String,
}

#[derive(Debug, Deserialize)]
pub struct DeleteTravelComment {
    pub password: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TravelReactionResponse {
    pub emoji: String,
    pub count: i32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TravelCommentResponse {
    pub id: i32,
    pub author: String,
    pub content: String,
    pub created_at: NaiveDateTime,
    pub reactions: Vec<TravelReactionResponse>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TravelCommentsResponse {
    pub comments: Vec<TravelCommentResponse>,
}

#[derive(Debug, Deserialize)]
pub struct TravelReactionRequest {
    pub emoji: String,
}
