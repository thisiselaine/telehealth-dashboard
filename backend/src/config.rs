use serde::Deserialize;
use std::env;

#[derive(Deserialize)]
pub struct Config {
    pub database_url: String,
    pub server_addr: String,
    pub jwt_secret: String,
}

pub fn get_config() -> Config {
    dotenv::dotenv().ok();
    Config {
        database_url: env::var("DATABASE_URL").expect("DATABASE_URL must be set"),
        server_addr: env::var("SERVER_ADDR").unwrap_or_else(|_| "127.0.0.1:8080".to_string()),
        jwt_secret: env::var("JWT_SECRET").expect("JWT_SECRET must be set"),
    }
}
