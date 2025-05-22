use actix_web::web;
use crate::blog::handlers;

pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.service(handlers::list_posts)
        .service(handlers::get_post)
        .service(handlers::create_post)
        .service(handlers::update_post)
        .service(handlers::delete_post);
}
