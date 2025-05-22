use actix_web::{ get, post, put, delete, web, HttpResponse, Responder, ResponseError, HttpRequest };

use crate::errors::ServiceError;
use crate::blog::dto::{ CreatePost, UpdatePost };
use crate::blog::service;
use crate::db::DbPool;
use crate::config::AppConfig;
use crate::user::model::{ User };
use crate::user::handlers::{ require_admin };

#[get("/posts")]
pub async fn list_posts(req: HttpRequest, pool: web::Data<DbPool>) -> impl Responder {
    println!("▶️ list_posts 호출: {} {}", req.method(), req.uri());

    match service::list_all(&pool).await {
        Ok(posts) => {
            println!("✅ list_posts 반환: {}개 포스트", posts.len());
            HttpResponse::Ok().json(posts)
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
