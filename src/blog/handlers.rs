use actix_web::{ get, post, put, delete, web, HttpResponse, Responder, ResponseError, HttpRequest };
use serde::{ Deserialize };

use crate::errors::ServiceError;
use crate::blog::dto::{ CreatePost, UpdatePost, BlurRequest, BlurResponse };
use crate::blog::service;
use crate::db::DbPool;
use crate::config::AppConfig;
use crate::user::model::{ User };
use crate::user::handlers::{ require_admin };

#[derive(Debug, Deserialize)]
struct Pagination {
    page: Option<u32>,
}

#[post("/posts/blur")]
pub async fn blur_image(web::Json(dto): web::Json<BlurRequest>) -> impl Responder {
    match service::blur_image(&dto.url).await {
        Ok(data_url) => { HttpResponse::Ok().json(BlurResponse { data_url }) }
        Err(e) => { e.error_response() }
    }
}

#[get("/posts")]
pub async fn list_posts(
    pool: web::Data<DbPool>,
    web::Query(pagination): web::Query<Pagination>
) -> impl Responder {
    let page_size = 12;

    let (limit, offset) = if let Some(page_num) = pagination.page {
        let page_num = page_num.max(1);
        let offset = ((page_num as i64) - 1) * page_size;
        (page_size, offset)
    } else {
        (1000, 0)
    };

    match service::list_all(&pool, limit, offset).await {
        Ok(data) => { HttpResponse::Ok().json(data) }
        Err(e) => { e.error_response() }
    }
}

#[get("/posts/{id}")]
pub async fn get_post(pool: web::Data<DbPool>, path: web::Path<i32>) -> impl Responder {
    let id = path.into_inner();

    match service::get_by_id(&pool, id).await {
        Ok(post) => { HttpResponse::Ok().json(post) }
        Err(e) => { e.error_response() }
    }
}

#[post("/posts")]
pub async fn create_post(
    req: HttpRequest,
    cfg: web::Data<AppConfig>,
    pool: web::Data<DbPool>,
    web::Json(dto): web::Json<CreatePost>
) -> impl Responder {
    let auth_header = req
        .headers()
        .get("Authorization")
        .and_then(|h| h.to_str().ok());

    let user = User::from_basic_auth(auth_header, &cfg.admin_user, &cfg.admin_pass);
    if !require_admin(&user) {
        return ServiceError::Unauthorized.error_response();
    }

    match service::create(&pool, dto).await {
        Ok(post) => { HttpResponse::Created().json(post) }
        Err(e) => { e.error_response() }
    }
}

#[put("/posts/{id}")]
pub async fn update_post(
    req: HttpRequest,
    cfg: web::Data<AppConfig>,
    pool: web::Data<DbPool>,
    path: web::Path<i32>,
    web::Json(dto): web::Json<UpdatePost>
) -> impl Responder {
    let id = path.into_inner();

    let auth_header = req
        .headers()
        .get("Authorization")
        .and_then(|h| h.to_str().ok());

    let user = User::from_basic_auth(auth_header, &cfg.admin_user, &cfg.admin_pass);
    if !require_admin(&user) {
        return ServiceError::Unauthorized.error_response();
    }

    match service::update(&pool, id, dto).await {
        Ok(post) => { HttpResponse::Ok().json(post) }
        Err(e) => { e.error_response() }
    }
}

#[delete("/posts/{id}")]
pub async fn delete_post(
    req: HttpRequest,
    cfg: web::Data<AppConfig>,
    pool: web::Data<DbPool>,
    path: web::Path<i32>
) -> impl Responder {
    let id = path.into_inner();

    let auth_header = req
        .headers()
        .get("Authorization")
        .and_then(|h| h.to_str().ok());

    let user = User::from_basic_auth(auth_header, &cfg.admin_user, &cfg.admin_pass);
    if !require_admin(&user) {
        return ServiceError::Unauthorized.error_response();
    }

    match service::delete(&pool, id).await {
        Ok(_) => { HttpResponse::NoContent().finish() }
        Err(e) => { e.error_response() }
    }
}
