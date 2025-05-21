use actix_web::{ web, App, HttpServer };
use confik::{ Configuration as _, EnvSource };
use dotenvy::dotenv;

use crate::config::AppConfig;

mod config;
mod db;
mod user;
mod blog;
mod errors;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();

    let config = AppConfig::builder().override_with(EnvSource::new()).try_build().unwrap();

    let pool = db::init_pool(&config.pg);

    let bind_addr = config.server_addr.clone();

    let server = HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(config.clone()))
            .app_data(web::Data::new(pool.clone()))
            .configure(user::routes::init)
            .configure(blog::routes::init)
    }).bind(&bind_addr)?;
    println!("🚀 Server running at http://localhost:8080");
    server.run().await
}
