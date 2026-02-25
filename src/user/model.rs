use serde::{ Deserialize, Serialize };
use base64::{ engine::general_purpose::STANDARD, Engine as _ };
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

    pub fn from_basic_auth(auth_header: Option<&str>, admin_user: &str, admin_pass: &str) -> Self {
        if let Some(header) = auth_header {
            if let Some(b64) = header.strip_prefix("Bearer ") {
                if let Ok(decoded) = decode_padded(b64) {
                    if let Ok(cred) = String::from_utf8(decoded) {
                        let mut parts = cred.splitn(2, ':');
                        if parts.next() == Some(admin_user) && parts.next() == Some(admin_pass) {
                            return User {
                                username: admin_user.to_string(),
                                role: Role::Admin,
                            };
                        }
                    }
                }
            }
        }
        User {
            username: String::new(),
            role: Role::Guest,
        }
    }
}

fn decode_padded(b64: &str) -> Result<Vec<u8>, base64::DecodeError> {
    let mut s = b64.to_string();
    let rem = s.len() % 4;
    if rem != 0 {
        s += &"=".repeat(4 - rem);
    }
    STANDARD.decode(&s)
}
