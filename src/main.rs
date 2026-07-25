use actix_cors::Cors;
use actix_web::{App, HttpServer, http::header, middleware::Logger, web};
use confik::{Configuration as _, EnvSource};
use dotenvy::dotenv;
use env_logger::Env;

use crate::config::AppConfig;

mod blog;
mod config;
mod db;
mod errors;
mod travel;
mod user;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();

    env_logger::init_from_env(Env::default().default_filter_or("info"));

    let config = AppConfig::builder()
        .override_with(EnvSource::new())
        .try_build()
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidInput, e.to_string()))?;
    let pool = db::init_pool(&config.pg);
    let bind_addr = config.server_addr.clone();

    let server = HttpServer::new(move || {
        let mut cors = Cors::default()
            .allowed_methods(vec!["GET", "POST", "PUT", "DELETE", "OPTIONS"])
            .allowed_headers(vec![header::CONTENT_TYPE])
            .supports_credentials()
            .max_age(3600);

        for origin in config
            .cors_allowed_origins
            .split(',')
            .map(str::trim)
            .filter(|origin| !origin.is_empty())
        {
            cors = cors.allowed_origin(origin);
        }

        App::new()
            .wrap(cors)
            .wrap(Logger::default())
            .app_data(web::Data::new(config.clone()))
            .app_data(web::Data::new(pool.clone()))
            .configure(user::routes::init)
            .configure(blog::routes::init)
            .configure(travel::routes::init)
    })
    .bind(&bind_addr)?;
    tracing::info!("server running at http://{bind_addr}");
    server.run().await
}
