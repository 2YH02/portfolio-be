use actix_web::web;
use crate::user::handlers;

pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.service(handlers::me);
}
