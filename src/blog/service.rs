use crate::db::DbPool;
use crate::blog::model::Post;
use crate::blog::dto::{ CreatePost, UpdatePost };
use crate::errors::ServiceError;

use tokio_pg_mapper::FromTokioPostgresRow;

pub async fn list_all(pool: &DbPool) -> Result<Vec<Post>, ServiceError> {
    let client = pool.get().await.map_err(|_| ServiceError::InternalServerError)?;

    let stmt = client
        .prepare(
            "SELECT id, title, body, tags, thumbnail, created_at FROM posts ORDER BY created_at DESC"
        ).await
        .map_err(|_| ServiceError::InternalServerError)?;

    let rows = client.query(&stmt, &[]).await.map_err(|_| ServiceError::InternalServerError)?;

    let posts = rows
        .into_iter()
        .map(|row| Post::from_row_ref(&row).unwrap())
        .collect();

    Ok(posts)
}

pub async fn get_by_id(pool: &DbPool, post_id: i32) -> Result<Post, ServiceError> {
    let client = pool.get().await.map_err(|_| ServiceError::InternalServerError)?;

    let stmt = client
        .prepare("SELECT id, title, body, tags, thumbnail, created_at FROM posts WHERE id = $1").await
        .map_err(|_| ServiceError::InternalServerError)?;

    let row = client.query_one(&stmt, &[&post_id]).await.map_err(|_| ServiceError::NotFound)?;

    Ok(Post::from_row_ref(&row).unwrap())
}

pub async fn create(pool: &DbPool, dto: CreatePost) -> Result<Post, ServiceError> {
    if dto.title.trim().is_empty() {
        return Err(ServiceError::BadRequest("제목을 입력해주세요".into()));
    }
    if dto.body.trim().is_empty() {
        return Err(ServiceError::BadRequest("본문을 입력해주세요".into()));
    }
    if dto.thumbnail.trim().is_empty() {
        return Err(ServiceError::BadRequest("대표 이미지를 설정해주세요".into()));
    }

    let client = pool.get().await.map_err(|_| ServiceError::InternalServerError)?;

    let stmt = client
        .prepare(
            "INSERT INTO posts (title, body, tags, thumbnail) \
         VALUES ($1, $2, $3, $4) \
         RETURNING id, title, body, tags, thumbnail, created_at"
        ).await
        .map_err(|_| ServiceError::InternalServerError)?;

    let row = client
        .query_one(&stmt, &[&dto.title, &dto.body, &dto.tags, &dto.thumbnail]).await
        .map_err(|_| ServiceError::InternalServerError)?;

    Ok(Post::from_row_ref(&row).unwrap())
}

pub async fn update(pool: &DbPool, post_id: i32, dto: UpdatePost) -> Result<Post, ServiceError> {
    let client = pool.get().await.map_err(|_| ServiceError::InternalServerError)?;

    let stmt = client
        .prepare(
            "\
        UPDATE posts SET \
            title = COALESCE($1, title), \
            body  = COALESCE($2, body) \
        WHERE id = $3 \
        RETURNING id, title, body, tags, thumbnail, created_at"
        ).await
        .map_err(|_| ServiceError::InternalServerError)?;

    let row = client
        .query_one(&stmt, &[&dto.title, &dto.body, &post_id]).await
        .map_err(|_| ServiceError::NotFound)?;

    Ok(Post::from_row_ref(&row).unwrap())
}

pub async fn delete(pool: &DbPool, post_id: i32) -> Result<(), ServiceError> {
    let client = pool.get().await.map_err(|_| ServiceError::InternalServerError)?;

    let stmt = client
        .prepare("DELETE FROM posts WHERE id = $1").await
        .map_err(|_| ServiceError::InternalServerError)?;

    client.execute(&stmt, &[&post_id]).await.map_err(|_| ServiceError::InternalServerError)?;

    Ok(())
}
