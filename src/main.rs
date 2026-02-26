use actix_web::{ web, App, HttpServer, middleware::Logger };
use confik::{ Configuration as _, EnvSource };
use dotenvy::dotenv;
use env_logger::Env;

use crate::config::AppConfig;

mod config;
mod db;
mod user;
mod blog;
mod errors;

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
        App::new()
            .wrap(Logger::default())
            .app_data(web::Data::new(config.clone()))
            .app_data(web::Data::new(pool.clone()))
            .configure(user::routes::init)
            .configure(blog::routes::init)
    }).bind(&bind_addr)?;
    tracing::info!("server running at http://{bind_addr}");
    server.run().await
}
