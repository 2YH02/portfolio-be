use serde::{ Deserialize, Serialize };
use jsonwebtoken::{ decode, DecodingKey, Validation };

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub role: String,
    pub exp: usize,
}

#[derive(Debug, Serialize, PartialEq, Eq)]
pub enum Role {
    Admin,
    Guest,
}

#[derive(Debug, Serialize)]
pub struct User {
    pub username: String,
    pub role: Role,
}

impl User {
    pub fn from_jwt(token: &str, jwt_secret: &str) -> Self {
        let key = DecodingKey::from_secret(jwt_secret.as_bytes());
        match decode::<Claims>(token, &key, &Validation::default()) {
            Ok(data) => {
                let role = if data.claims.role == "Admin" { Role::Admin } else { Role::Guest };
                User { username: data.claims.sub, role }
            }
            Err(_) => User { username: String::new(), role: Role::Guest },
        }
    }

}
