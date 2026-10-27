use actix_web::{web, HttpRequest, HttpResponse, ResponseError};
use crate::{db::DbPool, errors::ServiceError};
use super::service;

async fn respond(pool: web::Data<DbPool>, path: web::Path<i32>, req: HttpRequest, change: Option<bool>) -> Result<HttpResponse, ServiceError> {
    let hash = match req.cookie("portfolio_like_client") {
        Some(cookie) => service::client_hash(cookie.value())?,
        None if change.is_none() => String::new(),
        None => return Err(ServiceError::BadRequest("client token required".into())),
    };
    let result = service::likes(&pool, path.into_inner(), &hash, change).await?;
    Ok(HttpResponse::Ok().insert_header(("Cache-Control", "no-store")).json(result))
}

pub async fn get(pool: web::Data<DbPool>, path: web::Path<i32>, req: HttpRequest) -> HttpResponse {
    respond(pool, path, req, None).await.unwrap_or_else(|e| e.error_response())
}
pub async fn like(pool: web::Data<DbPool>, path: web::Path<i32>, req: HttpRequest) -> HttpResponse {
    respond(pool, path, req, Some(true)).await.unwrap_or_else(|e| e.error_response())
}
pub async fn unlike(pool: web::Data<DbPool>, path: web::Path<i32>, req: HttpRequest) -> HttpResponse {
    respond(pool, path, req, Some(false)).await.unwrap_or_else(|e| e.error_response())
}
