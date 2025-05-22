use serde::{ Serialize, Deserialize };
use chrono::NaiveDateTime;
use tokio_pg_mapper_derive::PostgresMapper;

#[derive(Debug, Serialize, Deserialize, PostgresMapper)]
#[pg_mapper(table = "posts")]
pub struct Post {
    pub id: i32,
    pub title: String,
    pub description: String,
    pub body: String,
    pub tags: Vec<String>,
    pub thumbnail: String,
    pub created_at: NaiveDateTime,
}
