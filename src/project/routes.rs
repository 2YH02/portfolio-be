use actix_web::web;
use super::handlers;

pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.route("/projects/{id}/likes", web::get().to(handlers::get))
        .route("/projects/{id}/like", web::post().to(handlers::like))
        .route("/projects/{id}/like", web::delete().to(handlers::unlike));
}
