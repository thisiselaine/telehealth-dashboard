use actix_web::{App, HttpServer, web};
use dotenv::dotenv;
use std::env;

mod config;
mod db;
mod routes;
mod services;
mod utils;
mod middleware;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    let config = config::get_config();
    let pool = db::connection::establish_connection(&config.database_url).await.unwrap();

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(pool.clone()))
            .wrap(middleware::auth_middleware::AuthMiddleware)
            .wrap(middleware::logging_middleware::LoggingMiddleware)
            .configure(routes::init_routes)
    })
    .bind(config.server_addr)?
    .run()
    .await
}
