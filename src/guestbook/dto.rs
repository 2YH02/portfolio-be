use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct CreateGuestbookEntry {
    pub nickname: String,
    pub message: String,
    #[serde(default)]
    pub website: String,
}

#[derive(Debug, Serialize)]
pub struct GuestbookEntryResponse {
    pub id: i32,
    pub scope: String,
    pub nickname: String,
    pub message: String,
    pub created_at: NaiveDateTime,
}

#[derive(Debug, Serialize)]
pub struct GuestbookListResponse {
    pub entries: Vec<GuestbookEntryResponse>,
}

#[derive(Debug, Deserialize)]
pub struct ModerateGuestbookEntry {
    pub status: String,
    pub reason: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct GuestbookModerationEntry {
    pub id: i32,
    pub scope: String,
    pub nickname: String,
    pub message: String,
    pub status: String,
    pub moderation_reason: Option<String>,
    pub created_at: NaiveDateTime,
}
