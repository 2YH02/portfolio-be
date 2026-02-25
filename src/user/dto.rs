use serde::{ Deserialize, Serialize };

#[derive(Debug, Deserialize)]
pub struct MeRequest {
    pub user: String,
    pub password: String,
}

#[derive(Debug, Serialize)]
pub struct AuthResponse {
    pub username: String,
    pub role: String,
}
