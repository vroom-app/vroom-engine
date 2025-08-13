use std::time::Duration;
use crate::shared::error::Result;
use sqlx::PgPool;

pub async fn create_pool(database_url: &str) -> Result<PgPool> {
    let pool = sqlx::postgres::PgPoolOptions::new()
        .max_connections(10)
        .acquire_timeout(Duration::from_secs(30))
        .connect(database_url)
        .await?;

    tracing::info!("Pool was initialized.");

    let _ = sqlx::query("SELECT 1").fetch_one(&pool).await?;

    tracing::info!("Database connection was successfully established.");
    Ok(pool)
}