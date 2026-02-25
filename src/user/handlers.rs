use actix_web::{ get, post, HttpRequest, HttpResponse, Responder, web };
use crate::config::AppConfig;
use crate::user::model::{ User, Role };
use crate::user::dto::{ MeRequest };

#[get("/me")]
pub async fn me(req: HttpRequest, cfg: web::Data<AppConfig>) -> impl Responder {
    tracing::debug!("{} {}", req.method(), req.uri());

    let auth_header = req
        .headers()
        .get("Authorization")
        .and_then(|hv| hv.to_str().ok());

    let user = User::from_basic_auth(auth_header, &cfg.admin_user, &cfg.admin_pass);
    tracing::debug!("auth result: role={:?}", user.role);

    HttpResponse::Ok().json(user)
}

#[post("/auth")]
pub async fn auth(web::Json(dto): web::Json<MeRequest>) -> impl Responder {
    tracing::debug!("auth attempt: user={}", dto.user);

    let admin_user = std::env::var("ADMIN_USER").unwrap_or_default();
    let admin_pass = std::env::var("ADMIN_PASS").unwrap_or_default();

    let resp_user = if dto.user == admin_user && dto.password == admin_pass {
        User {
            username: dto.user.clone(),
            role: Role::Admin,
        }
    } else {
        User {
            username: String::new(),
            role: Role::Guest,
        }
    };

    HttpResponse::Ok().json(resp_user)
}

pub fn require_admin(user: &User) -> bool {
    user.role == Role::Admin
}
