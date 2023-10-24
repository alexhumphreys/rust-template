use dotenv::dotenv;
use sqlx::{postgres::PgPoolOptions, Pool, Postgres};
use std::sync::Arc;

pub type Db = Arc<Pool<Postgres>>;

pub async fn db_connect() -> Pool<Postgres> {
    dotenv().ok();
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let pool = match PgPoolOptions::new()
        .max_connections(10)
        .acquire_timeout(std::time::Duration::from_secs(2))
        .test_before_acquire(false)
        .connect(&database_url)
        .await
    {
        Ok(pool) => {
            tracing::info!("✅Connection to the database is successful!");
            pool
        }
        Err(err) => {
            tracing::error!("🔥 Failed to connect to the database: {:?}", err);
            std::process::exit(1);
        }
    };
    pool
}
