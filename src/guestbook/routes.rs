use actix_web::web;

use crate::guestbook::handlers;

pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.service(handlers::list)
        .service(handlers::create)
        .service(handlers::list_admin)
        .service(handlers::moderate)
        .service(handlers::delete);
}
