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
    println!("▶️ blur_image_handler 호출: url={}", dto.url);

    match service::blur_image(&dto.url).await {
        Ok(data_url) => {
            println!("✅ blur_image 성공: url={}, length={}", dto.url, data_url.len());
            HttpResponse::Ok().json(BlurResponse { data_url })
        }
        Err(e) => {
            println!("⚠️ blur_image 실패: {:?}", e);
            e.error_response()
        }
    }
}

#[get("/posts")]
pub async fn list_posts(
    req: HttpRequest,
    pool: web::Data<DbPool>,
    web::Query(pagination): web::Query<Pagination>
) -> impl Responder {
    println!("▶️ list_posts 호출: {} {}", req.method(), req.uri());

    let page_num = pagination.page.unwrap_or(1).max(1);
    let page_size = 12;
    let offset = ((page_num as i64) - 1) * page_size;

    println!("✅ page_num: {}, page_size: {}, offset: {}", page_num, page_size, offset);

    match service::list_all(&pool, page_size, offset).await {
        Ok(data) => {
            println!("✅ list_posts 반환: {}개 포스트", data.posts.len());
            HttpResponse::Ok().json(data)
        }
        Err(e) => {
            println!("⚠️ list_posts 에러: {:?}", e);
            e.error_response()
        }
    }
}

#[get("/posts/{id}")]
pub async fn get_post(
    req: HttpRequest,
    pool: web::Data<DbPool>,
    path: web::Path<i32>
) -> impl Responder {
    let id = path.into_inner();
    println!("▶️ get_post 호출: {} {} (id={})", req.method(), req.uri(), id);

    match service::get_by_id(&pool, id).await {
        Ok(post) => {
            println!("✅ get_post 성공: {:?}", post);
            HttpResponse::Ok().json(post)
        }
        Err(e) => {
            println!("⚠️ get_post 에러: {:?}", e);
            e.error_response()
        }
    }
}

#[post("/posts")]
pub async fn create_post(
    req: HttpRequest,
    cfg: web::Data<AppConfig>,
    pool: web::Data<DbPool>,
    web::Json(dto): web::Json<CreatePost>
) -> impl Responder {
    println!("▶️ create_post 호출: {:?}", dto);

    let auth_header = req
        .headers()
        .get("Authorization")
        .and_then(|h| h.to_str().ok());
    println!("📍 Authorization 헤더: {:?}", auth_header);

    let user = User::from_basic_auth(auth_header, &cfg.admin_user, &cfg.admin_pass);
    if !require_admin(&user) {
        return ServiceError::Unauthorized.error_response();
    }
    println!("✅ 인증 통과: user={:?}", user);

    println!("📍 DB insert 시작");
    match service::create(&pool, dto).await {
        Ok(post) => {
            println!("✅ 포스트 생성 성공: {:?}", post);
            HttpResponse::Created().json(post)
        }
        Err(e) => {
            println!("⚠️ 포스트 생성 실패: {:?}", e);
            e.error_response()
        }
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
    println!("▶️ update_post 호출: {} {} (id={}), payload: {:?}", req.method(), req.uri(), id, dto);

    let auth_header = req
        .headers()
        .get("Authorization")
        .and_then(|h| h.to_str().ok());
    println!("📍 Authorization 헤더: {:?}", auth_header);

    let user = User::from_basic_auth(auth_header, &cfg.admin_user, &cfg.admin_pass);
    if !require_admin(&user) {
        println!("❌ 인증 실패: user={:?}", user);
        return ServiceError::Unauthorized.error_response();
    }
    println!("✅ 인증 통과: user={:?}", user);

    match service::update(&pool, id, dto).await {
        Ok(post) => {
            println!("✅ update_post 성공: {:?}", post);
            HttpResponse::Ok().json(post)
        }
        Err(e) => {
            println!("⚠️ update_post 에러: {:?}", e);
            e.error_response()
        }
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
    println!("▶️ delete_post 호출: {} {} (id={})", req.method(), req.uri(), id);

    let auth_header = req
        .headers()
        .get("Authorization")
        .and_then(|h| h.to_str().ok());
    println!("📍 Authorization 헤더: {:?}", auth_header);

    let user = User::from_basic_auth(auth_header, &cfg.admin_user, &cfg.admin_pass);
    if !require_admin(&user) {
        println!("❌ 인증 실패: user={:?}", user);
        return ServiceError::Unauthorized.error_response();
    }
    println!("✅ 인증 통과: user={:?}", user);

    match service::delete(&pool, id).await {
        Ok(_) => {
            println!("✅ delete_post 성공: id={} 삭제 완료", id);
            HttpResponse::NoContent().finish()
        }
        Err(e) => {
            println!("⚠️ delete_post 에러: {:?}", e);
            e.error_response()
        }
    }
}
