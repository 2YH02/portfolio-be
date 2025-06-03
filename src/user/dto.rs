use serde::{ Deserialize };

#[derive(Debug, Deserialize)]
pub struct MeRequest {
    pub user: String,
    pub password: String,
}
