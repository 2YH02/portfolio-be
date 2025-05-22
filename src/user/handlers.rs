use actix_web::{ get, HttpRequest, HttpResponse, Responder, web };
use crate::config::AppConfig;
use crate::user::model::{ User, Role };

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

pub fn require_admin(user: &User) -> bool {
    user.role == Role::Admin
}
