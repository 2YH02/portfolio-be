use actix_web::web;

use crate::travel::handlers;

pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.service(handlers::get_likes)
        .service(handlers::like_post)
        .service(handlers::unlike_post)
        .service(handlers::list_comments)
        .service(handlers::create_comment)
        .service(handlers::delete_comment)
        .service(handlers::react_comment);
}
