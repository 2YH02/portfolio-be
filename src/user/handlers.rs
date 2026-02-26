use actix_web::{ get, post, HttpRequest, HttpResponse, Responder, web };
use actix_web::cookie::{ Cookie, SameSite };
use jsonwebtoken::{ encode, EncodingKey, Header };
use std::time::{ SystemTime, UNIX_EPOCH };

use crate::config::AppConfig;
use crate::user::model::{ Claims, User, Role };
use crate::user::dto::{ AuthResponse, MeRequest };

pub const AUTH_COOKIE: &str = "admin_token";

pub fn auth_from_cookie(req: &HttpRequest, cfg: &AppConfig) -> User {
    match req.cookie(AUTH_COOKIE) {
        Some(c) => User::from_jwt(c.value(), &cfg.jwt_secret),
        None => User { username: String::new(), role: Role::Guest },
    }
}

#[get("/me")]
pub async fn me(req: HttpRequest, cfg: web::Data<AppConfig>) -> impl Responder {
    tracing::debug!("{} {}", req.method(), req.uri());

    let user = auth_from_cookie(&req, &cfg);
    tracing::debug!("auth result: role={:?}", user.role);

    HttpResponse::Ok().json(user)
}

#[post("/auth")]
pub async fn auth(
    web::Json(dto): web::Json<MeRequest>,
    cfg: web::Data<AppConfig>
) -> impl Responder {
    tracing::debug!("auth attempt: user={}", dto.user);

    if dto.user != cfg.admin_user || dto.password != cfg.admin_pass {
        return HttpResponse::Unauthorized().finish();
    }

    let exp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as usize + 86_400;

    let claims = Claims { sub: dto.user.clone(), role: "Admin".to_string(), exp };

    let token = match encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(cfg.jwt_secret.as_bytes()),
    ) {
        Ok(t) => t,
        Err(e) => {
            tracing::error!("JWT encode error: {e}");
            return HttpResponse::InternalServerError().finish();
        }
    };

    let same_site = if cfg.cookie_secure { SameSite::None } else { SameSite::Lax };
    let cookie = Cookie::build(AUTH_COOKIE, token)
        .http_only(true)
        .secure(cfg.cookie_secure)
        .same_site(same_site)
        .path("/")
        .max_age(actix_web::cookie::time::Duration::seconds(86_400))
        .finish();

    HttpResponse::Ok()
        .cookie(cookie)
        .json(AuthResponse { username: dto.user, role: "Admin".to_string() })
}

pub fn require_admin(user: &User) -> bool {
    user.role == Role::Admin
}
