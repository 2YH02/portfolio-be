use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use tokio_pg_mapper_derive::PostgresMapper;

#[derive(Debug, Serialize, Deserialize, PostgresMapper)]
#[pg_mapper(table = "guestbook_entries")]
pub struct GuestbookEntry {
    pub id: i32,
    pub scope: String,
    pub nickname: String,
    pub message: String,
    pub status: String,
    pub moderation_reason: Option<String>,
    pub client_hash: String,
    pub created_at: NaiveDateTime,
}
