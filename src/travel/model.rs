use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use tokio_pg_mapper_derive::PostgresMapper;

#[derive(Debug, Serialize, Deserialize, PostgresMapper)]
#[pg_mapper(table = "travel_comments")]
pub struct TravelComment {
    pub id: i32,
    pub author: String,
    pub content: String,
    pub created_at: NaiveDateTime,
}

#[derive(Debug, Serialize, Deserialize, PostgresMapper)]
#[pg_mapper(table = "travel_reactions")]
pub struct TravelReaction {
    pub emoji: String,
    pub count: i32,
}
