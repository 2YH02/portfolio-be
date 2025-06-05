use tokio_pg_mapper::FromTokioPostgresRow;
use reqwest::Client;
use image::{ imageops::blur, DynamicImage };
use image::codecs::jpeg::JpegEncoder;
use base64::engine::general_purpose::STANDARD;
use base64::Engine;

use crate::db::DbPool;
use crate::blog::model::Post;
use crate::blog::dto::{ CreatePost, UpdatePost, PostListResponse };
use crate::errors::ServiceError;

pub async fn blur_image(url: &str) -> Result<String, ServiceError> {
    let response = Client::new()
        .get(url)
        .send().await
        .map_err(|e| ServiceError::BadRequest(format!("이미지 다운로드 실패: {}", e)))?;
    let bytes = response
        .bytes().await
        .map_err(|e| ServiceError::BadRequest(format!("이미지 바이트 읽기 실패: {}", e)))?;

    let img: DynamicImage = image
        ::load_from_memory(&bytes)
        .map_err(|e| ServiceError::InternalServerError(e.to_string()))?;
    let blurred = blur(&img, 10.0);

    let mut buf = Vec::new();
    let mut encoder = JpegEncoder::new_with_quality(&mut buf, 60);
    encoder.encode_image(&blurred).map_err(|e| ServiceError::InternalServerError(e.to_string()))?;

    let b64 = STANDARD.encode(&buf);
    Ok(format!("data:image/jpeg;base64,{}", b64))
}

pub async fn list_all(
    pool: &DbPool,
    limit: i64,
    offset: i64
) -> Result<PostListResponse, ServiceError> {
    let client = pool.get().await.map_err(|e| ServiceError::InternalServerError(e.to_string()))?;

    let count_stmt = client
        .prepare("SELECT COUNT(*) FROM posts").await
        .map_err(|e| ServiceError::InternalServerError(e.to_string()))?;

    let count_row = client
        .query_one(&count_stmt, &[]).await
        .map_err(|e| ServiceError::InternalServerError(e.to_string()))?;

    let total_count: i64 = count_row.get(0);

    let stmt = client
        .prepare(
            "SELECT id, title, description, body, tags, thumbnail, thumbnail_blur, created_at
             FROM posts
             ORDER BY created_at DESC
             OFFSET $1
             LIMIT  $2"
        ).await
        .map_err(|e| ServiceError::InternalServerError(e.to_string()))?;

    let rows = client
        .query(&stmt, &[&offset, &limit]).await
        .map_err(|e| ServiceError::InternalServerError(e.to_string()))?;

    let posts = rows
        .into_iter()
        .map(|row| Post::from_row_ref(&row).unwrap())
        .collect();

    Ok(PostListResponse {
        total_count,
        posts,
    })
}

pub async fn get_by_id(pool: &DbPool, post_id: i32) -> Result<Post, ServiceError> {
    let client = pool.get().await.map_err(|e| ServiceError::InternalServerError(e.to_string()))?;

    let stmt = client
        .prepare(
            "SELECT id, title, description, body, tags, thumbnail, thumbnail_blur, created_at FROM posts WHERE id = $1"
        ).await
        .map_err(|e| ServiceError::InternalServerError(e.to_string()))?;

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

    let client = pool.get().await.map_err(|e| ServiceError::InternalServerError(e.to_string()))?;

    let stmt = client
        .prepare(
            "INSERT INTO posts (title, description, body, tags, thumbnail, thumbnail_blur) \
         VALUES ($1, $2, $3, $4, $5, $6) \
         RETURNING id, title, description, body, tags, thumbnail, thumbnail_blur, created_at"
        ).await
        .map_err(|e| ServiceError::InternalServerError(e.to_string()))?;

    let row = client
        .query_one(
            &stmt,
            &[
                &dto.title,
                &dto.description,
                &dto.body,
                &dto.tags,
                &dto.thumbnail,
                &dto.thumbnail_blur,
            ]
        ).await
        .map_err(|e| ServiceError::InternalServerError(e.to_string()))?;

    Ok(Post::from_row_ref(&row).unwrap())
}

pub async fn update(pool: &DbPool, post_id: i32, dto: UpdatePost) -> Result<Post, ServiceError> {
    let client = pool.get().await.map_err(|e| ServiceError::InternalServerError(e.to_string()))?;

    let stmt = client
        .prepare(
            "\
        UPDATE posts SET \
            title = COALESCE($1, title), \
            description  = COALESCE($2, description), \
            body  = COALESCE($3, body) \
        WHERE id = $4 \
        RETURNING id, title, description, body, tags, thumbnail, thumbnail_blur, created_at"
        ).await
        .map_err(|e| ServiceError::InternalServerError(e.to_string()))?;

    let row = client
        .query_one(&stmt, &[&dto.title, &dto.description, &dto.body, &post_id]).await
        .map_err(|_| ServiceError::NotFound)?;

    Ok(Post::from_row_ref(&row).unwrap())
}

pub async fn delete(pool: &DbPool, post_id: i32) -> Result<(), ServiceError> {
    let client = pool.get().await.map_err(|e| ServiceError::InternalServerError(e.to_string()))?;

    let stmt = client
        .prepare("DELETE FROM posts WHERE id = $1").await
        .map_err(|e| ServiceError::InternalServerError(e.to_string()))?;

    client
        .execute(&stmt, &[&post_id]).await
        .map_err(|e| ServiceError::InternalServerError(e.to_string()))?;

    Ok(())
}
