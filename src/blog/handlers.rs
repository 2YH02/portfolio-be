use actix_web::{ get, post, put, delete, web, HttpRequest, HttpResponse, Responder, ResponseError };
use actix_web::cookie::{ Cookie, time::Duration };
use serde::{ Deserialize };

use crate::blog::dto::{ CreatePost, UpdatePost, BlurRequest, BlurResponse };
use crate::blog::service;
use crate::db::DbPool;
use crate::user::handlers::Admin;

#[derive(Debug, Deserialize)]
struct Pagination {
    page: Option<u32>,
    #[serde(rename = "pageSize")]
    page_size: Option<u32>,
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
    let page_size = pagination.page_size.unwrap_or(8).max(1) as i64;

    let (limit, offset) = if let Some(page_num) = pagination.page {
        let page_num = page_num.max(1);
        let offset = ((page_num as i64) - 1) * page_size;
        (page_size, offset)
    } else {
        (page_size, 0)
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

#[post("/posts/{id}/view")]
pub async fn view_post(pool: web::Data<DbPool>, path: web::Path<i32>, req: HttpRequest) -> impl Responder {
    let id = path.into_inner();

    let viewed_ids: Vec<i32> = req
        .cookie("viewed_posts")
        .map(|c| c.value().split(',').filter_map(|s| s.parse().ok()).collect())
        .unwrap_or_default();

    if viewed_ids.contains(&id) {
        return HttpResponse::NoContent().finish();
    }

    match service::increment_view(&pool, id).await {
        Ok(view_count) => {
            let mut new_ids = viewed_ids;
            new_ids.push(id);
            let cookie_value = new_ids.iter().map(|n| n.to_string()).collect::<Vec<_>>().join(",");
            let cookie = Cookie::build("viewed_posts", cookie_value)
                .max_age(Duration::days(1))
                .path("/")
                .finish();
            HttpResponse::Ok()
                .cookie(cookie)
                .json(serde_json::json!({ "view_count": view_count }))
        }
        Err(e) => { e.error_response() }
    }
}

#[get("/tags")]
pub async fn list_tags(pool: web::Data<DbPool>) -> impl Responder {
    match service::get_tags(&pool).await {
        Ok(tags) => { HttpResponse::Ok().json(tags) }
        Err(e) => { e.error_response() }
    }
}

#[get("/posts/popular")]
pub async fn popular_posts(pool: web::Data<DbPool>) -> impl Responder {
    match service::get_popular(&pool).await {
        Ok(posts) => { HttpResponse::Ok().json(posts) }
        Err(e) => { e.error_response() }
    }
}

#[post("/posts/{id}/like")]
pub async fn like_post(pool: web::Data<DbPool>, path: web::Path<i32>, req: HttpRequest) -> impl Responder {
    let id = path.into_inner();

    let liked_ids: Vec<i32> = req
        .cookie("liked_posts")
        .map(|c| c.value().split(',').filter_map(|s| s.parse().ok()).collect())
        .unwrap_or_default();

    if liked_ids.contains(&id) {
        return HttpResponse::NoContent().finish();
    }

    match service::increment_like(&pool, id).await {
        Ok(like_count) => {
            let mut new_ids = liked_ids;
            new_ids.push(id);
            let cookie_value = new_ids.iter().map(|n| n.to_string()).collect::<Vec<_>>().join(",");
            let cookie = Cookie::build("liked_posts", cookie_value)
                .max_age(Duration::days(7))
                .path("/")
                .finish();
            HttpResponse::Ok()
                .cookie(cookie)
                .json(serde_json::json!({ "like_count": like_count }))
        }
        Err(e) => { e.error_response() }
    }
}

#[post("/posts")]
pub async fn create_post(
    _: Admin,
    pool: web::Data<DbPool>,
    web::Json(dto): web::Json<CreatePost>
) -> impl Responder {
    match service::create(&pool, dto).await {
        Ok(post) => { HttpResponse::Created().json(post) }
        Err(e) => { e.error_response() }
    }
}

#[put("/posts/{id}")]
pub async fn update_post(
    _: Admin,
    pool: web::Data<DbPool>,
    path: web::Path<i32>,
    web::Json(dto): web::Json<UpdatePost>
) -> impl Responder {
    let id = path.into_inner();
    match service::update(&pool, id, dto).await {
        Ok(post) => { HttpResponse::Ok().json(post) }
        Err(e) => { e.error_response() }
    }
}

#[delete("/posts/{id}")]
pub async fn delete_post(
    _: Admin,
    pool: web::Data<DbPool>,
    path: web::Path<i32>
) -> impl Responder {
    let id = path.into_inner();
    match service::delete(&pool, id).await {
        Ok(_) => { HttpResponse::NoContent().finish() }
        Err(e) => { e.error_response() }
    }
}
