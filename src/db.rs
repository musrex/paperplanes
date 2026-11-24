use sqlx::{Pool, Sqlite, sqlite::SqlitePoolOptions};
use std::time::Duration;

pub type DbPool = Pool<Sqlite>;

pub async fn init_db(database_url: &str) -> Result<DbPool, sqlx::Error> {
    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .idle_timeout(Duration::from_secs(5))
        .connect(database_url)
        .await?;

    Ok(pool)
}
