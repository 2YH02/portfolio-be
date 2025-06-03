use actix_web::{ get, post, HttpRequest, HttpResponse, Responder, web };
use crate::config::AppConfig;
use crate::user::model::{ User, Role };
use crate::user::dto::{ MeRequest };

#[get("/me")]
pub async fn me(req: HttpRequest, cfg: web::Data<AppConfig>) -> impl Responder {
    println!("▶️ me 호출: {} {}", req.method(), req.uri());

    let auth_header = req
        .headers()
        .get("Authorization")
        .and_then(|hv| hv.to_str().ok());
    println!("📍 Authorization 헤더: {:?}", auth_header);

    let user = User::from_basic_auth(auth_header, &cfg.admin_user, &cfg.admin_pass);
    println!(
        "📍 파싱된 사용자 정보: {:?}, 어드민 유저: {:?}, 어드민 패스: {:?}",
        user,
        &cfg.admin_user,
        &cfg.admin_pass
    );

    HttpResponse::Ok().json(user)
}

#[post("/auth")]
pub async fn auth(web::Json(dto): web::Json<MeRequest>) -> impl Responder {
    println!("▶️ auth_handler 호출: user={}", dto.user);

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
