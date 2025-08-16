use sqlx::{Pool, Postgres, postgres::PgPoolOptions};
use std::time::Duration;

pub type DbPool = Pool<Postgres>;

pub async fn init_db(database_url: &str) -> Result<DbPool, sqlx::Error> {
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .idle_timeout(Duration::from_secs(5))
        .connect(database_url)
        .await?;

    Ok(pool)
}
