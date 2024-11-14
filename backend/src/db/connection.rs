use sqlx::{Pool, Mssql, mssql::MssqlPoolOptions};
use std::time::Duration;

pub type DbPool = Pool<Mssql>;

pub async fn establish_connection(database_url: &str) -> Result<DbPool, sqlx::Error> {
    MssqlPoolOptions::new()
        .max_connections(5)
        .connect_timeout(Duration::from_secs(5))
        .connect(database_url)
        .await
}
