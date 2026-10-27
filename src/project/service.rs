use sha2::{Digest, Sha256};
use crate::{db::DbPool, errors::ServiceError};
use super::dto::Likes;

pub fn validate_id(id: i32) -> Result<(), ServiceError> {
    if [0, 1, 4, 5, 6, 7].contains(&id) { Ok(()) } else { Err(ServiceError::NotFound) }
}

pub fn client_hash(token: &str) -> Result<String, ServiceError> {
    if token.len() != 36 || !token.bytes().enumerate().all(|(i, b)| {
        if [8, 13, 18, 23].contains(&i) { b == b'-' } else { b.is_ascii_hexdigit() }
    }) {
        return Err(ServiceError::BadRequest("invalid client token".into()));
    }
    Ok(format!("{:x}", Sha256::digest(token.as_bytes())))
}

pub async fn likes(pool: &DbPool, id: i32, hash: &str, change: Option<bool>) -> Result<Likes, ServiceError> {
    validate_id(id)?;
    let mut client = pool.get().await?;
    let tx = client.transaction().await?;
    match change {
        Some(true) => { tx.execute("INSERT INTO project_likes (project_id, client_hash) VALUES ($1, $2) ON CONFLICT DO NOTHING", &[&id, &hash]).await?; }
        Some(false) => { tx.execute("DELETE FROM project_likes WHERE project_id = $1 AND client_hash = $2", &[&id, &hash]).await?; }
        None => {}
    }
    let row = tx.query_one("SELECT COUNT(*)::bigint, COALESCE(bool_or(client_hash = $2), false) FROM project_likes WHERE project_id = $1", &[&id, &hash]).await?;
    let result = Likes { like_count: row.get(0), liked: row.get(1) };
    tx.commit().await?;
    Ok(result)
}
