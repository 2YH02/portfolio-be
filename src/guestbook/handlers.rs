use actix_web::{delete, get, post, web, HttpRequest, HttpResponse, Responder, ResponseError};

use crate::db::DbPool;
use crate::guestbook::dto::{CreateGuestbookEntry, ModerateGuestbookEntry};
use crate::guestbook::service;
use crate::user::handlers::Admin;

#[get("/guestbook/{scope}")]
pub async fn list(pool: web::Data<DbPool>, path: web::Path<String>) -> impl Responder {
    let scope = match service::validate_scope(&path.into_inner()) {
        Ok(scope) => scope,
        Err(error) => return error.error_response(),
    };
    match service::list_visible(&pool, &scope).await {
        Ok(entries) => {
            HttpResponse::Ok().json(crate::guestbook::dto::GuestbookListResponse { entries })
        }
        Err(error) => error.error_response(),
    }
}

#[post("/guestbook/{scope}")]
pub async fn create(
    pool: web::Data<DbPool>,
    path: web::Path<String>,
    req: HttpRequest,
    web::Json(dto): web::Json<CreateGuestbookEntry>,
) -> impl Responder {
    let scope = match service::validate_scope(&path.into_inner()) {
        Ok(scope) => scope,
        Err(error) => return error.error_response(),
    };
    let token = req
        .headers()
        .get("x-guestbook-token")
        .and_then(|value| value.to_str().ok())
        .filter(|value| !value.trim().is_empty())
        .or_else(|| req.connection_info().peer_addr())
        .unwrap_or("unknown");
    let client_hash = service::hash_client_token(token);
    match service::create(&pool, &scope, dto, &client_hash).await {
        Ok((entry, visible)) => HttpResponse::Created().json(serde_json::json!({
            "entry": entry,
            "visible": visible,
            "message": if visible { "방명록에 등록했어요" } else { "검수 후 공개돼요" }
        })),
        Err(error) => error.error_response(),
    }
}

#[get("/admin/guestbook")]
pub async fn list_admin(
    _: Admin,
    pool: web::Data<DbPool>,
    query: web::Query<std::collections::HashMap<String, String>>,
) -> impl Responder {
    match service::list_moderation(&pool, query.get("status").map(String::as_str)).await {
        Ok(entries) => HttpResponse::Ok().json(entries),
        Err(error) => error.error_response(),
    }
}

#[post("/admin/guestbook/{id}/moderate")]
pub async fn moderate(
    _: Admin,
    pool: web::Data<DbPool>,
    path: web::Path<i32>,
    web::Json(dto): web::Json<ModerateGuestbookEntry>,
) -> impl Responder {
    match service::moderate_entry(&pool, path.into_inner(), &dto.status, dto.reason.as_deref())
        .await
    {
        Ok(()) => HttpResponse::NoContent().finish(),
        Err(error) => error.error_response(),
    }
}

#[delete("/admin/guestbook/{id}")]
pub async fn delete(_: Admin, pool: web::Data<DbPool>, path: web::Path<i32>) -> impl Responder {
    match service::delete(&pool, path.into_inner()).await {
        Ok(()) => HttpResponse::NoContent().finish(),
        Err(error) => error.error_response(),
    }
}
