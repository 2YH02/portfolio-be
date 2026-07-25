use actix_web::cookie::{Cookie, SameSite, time::Duration};
use actix_web::{HttpRequest, HttpResponse, Responder, ResponseError, delete, get, post, web};

use crate::db::DbPool;
use crate::errors::ServiceError;
use crate::travel::dto::{
    CreateTravelComment, DeleteTravelComment, TravelCommentsResponse, TravelLikeResponse,
    TravelReactionRequest,
};
use crate::travel::service;

const LIKED_POSTS_COOKIE: &str = "travel_liked_posts";

#[get("/travel/posts/{slug}/likes")]
pub async fn get_likes(pool: web::Data<DbPool>, path: web::Path<String>) -> impl Responder {
    let slug = path.into_inner();

    match service::get_like_count(&pool, &slug).await {
        Ok(like_count) => HttpResponse::Ok().json(TravelLikeResponse { like_count }),
        Err(e) => e.error_response(),
    }
}

#[post("/travel/posts/{slug}/like")]
pub async fn like_post(
    pool: web::Data<DbPool>,
    path: web::Path<String>,
    req: HttpRequest,
) -> impl Responder {
    let slug = path.into_inner();
    let liked_slugs = cookie_values(&req, LIKED_POSTS_COOKIE);

    if liked_slugs.iter().any(|liked| liked == &slug) {
        return HttpResponse::NoContent().finish();
    }

    match service::increment_like(&pool, &slug).await {
        Ok(like_count) => {
            let mut new_slugs = liked_slugs;
            new_slugs.push(slug);
            let cookie = list_cookie(LIKED_POSTS_COOKIE, &new_slugs, Duration::days(7));

            HttpResponse::Ok()
                .cookie(cookie)
                .json(TravelLikeResponse { like_count })
        }
        Err(e) => e.error_response(),
    }
}

#[delete("/travel/posts/{slug}/like")]
pub async fn unlike_post(
    pool: web::Data<DbPool>,
    path: web::Path<String>,
    req: HttpRequest,
) -> impl Responder {
    let slug = path.into_inner();
    let liked_slugs = cookie_values(&req, LIKED_POSTS_COOKIE);

    if !liked_slugs.iter().any(|liked| liked == &slug) {
        return HttpResponse::NoContent().finish();
    }

    match service::decrement_like(&pool, &slug).await {
        Ok(like_count) => {
            let new_slugs = liked_slugs
                .into_iter()
                .filter(|liked| liked != &slug)
                .collect::<Vec<_>>();
            let cookie = list_cookie(LIKED_POSTS_COOKIE, &new_slugs, Duration::days(7));

            HttpResponse::Ok()
                .cookie(cookie)
                .json(TravelLikeResponse { like_count })
        }
        Err(e) => e.error_response(),
    }
}

#[get("/travel/posts/{slug}/comments")]
pub async fn list_comments(pool: web::Data<DbPool>, path: web::Path<String>) -> impl Responder {
    let slug = path.into_inner();

    match service::list_comments(&pool, &slug).await {
        Ok(comments) => HttpResponse::Ok().json(TravelCommentsResponse { comments }),
        Err(e) => e.error_response(),
    }
}

#[post("/travel/posts/{slug}/comments")]
pub async fn create_comment(
    pool: web::Data<DbPool>,
    path: web::Path<String>,
    web::Json(dto): web::Json<CreateTravelComment>,
) -> impl Responder {
    let slug = path.into_inner();

    match service::create_comment(&pool, &slug, dto).await {
        Ok(comment) => HttpResponse::Created().json(comment),
        Err(e) => e.error_response(),
    }
}

#[delete("/travel/comments/{id}")]
pub async fn delete_comment(
    pool: web::Data<DbPool>,
    path: web::Path<i32>,
    web::Json(dto): web::Json<DeleteTravelComment>,
) -> impl Responder {
    let id = path.into_inner();

    match service::delete_comment(&pool, id, &dto.password).await {
        Ok(_) => HttpResponse::NoContent().finish(),
        Err(e) => e.error_response(),
    }
}

#[post("/travel/comments/{id}/react")]
pub async fn react_comment(
    pool: web::Data<DbPool>,
    path: web::Path<i32>,
    req: HttpRequest,
    web::Json(dto): web::Json<TravelReactionRequest>,
) -> impl Responder {
    let id = path.into_inner();
    let Some(token) = service::emoji_cookie_token(&dto.emoji) else {
        return ServiceError::BadRequest("허용되지 않은 이모지입니다".into()).error_response();
    };

    let cookie_name = reaction_cookie_name(id);
    let reacted = cookie_values(&req, &cookie_name);

    if reacted.iter().any(|value| value == token) {
        return HttpResponse::NoContent().finish();
    }

    match service::increment_reaction(&pool, id, &dto.emoji).await {
        Ok(reaction) => {
            let mut new_reacted = reacted;
            new_reacted.push(token.to_string());
            let cookie = list_cookie(&cookie_name, &new_reacted, Duration::days(7));

            HttpResponse::Ok().cookie(cookie).json(reaction)
        }
        Err(e) => e.error_response(),
    }
}

fn reaction_cookie_name(comment_id: i32) -> String {
    format!("travel_reactions_{comment_id}")
}

fn cookie_values(req: &HttpRequest, name: &str) -> Vec<String> {
    req.cookie(name)
        .map(|c| {
            c.value()
                .split(',')
                .filter(|value| !value.is_empty())
                .map(ToOwned::to_owned)
                .collect()
        })
        .unwrap_or_default()
}

fn list_cookie(name: &str, values: &[String], max_age: Duration) -> Cookie<'static> {
    Cookie::build(name.to_string(), values.join(","))
        .http_only(true)
        .same_site(SameSite::Lax)
        .max_age(max_age)
        .path("/")
        .finish()
}
