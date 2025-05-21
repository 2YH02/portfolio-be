use actix_web::{ get, HttpRequest, HttpResponse, Responder, web };
use crate::config::AppConfig;
use crate::user::model::{ User, Role };

#[get("/me")]
pub async fn me(req: HttpRequest, cfg: web::Data<AppConfig>) -> impl Responder {
    let auth_header = req
        .headers()
        .get("Authorization")
        .and_then(|hv| hv.to_str().ok());
    let user = User::from_basic_auth(auth_header, &cfg.admin_user, &cfg.admin_pass);
    HttpResponse::Ok().json(user)
}

pub fn require_admin(user: &User) -> bool {
    user.role == Role::Admin
}
