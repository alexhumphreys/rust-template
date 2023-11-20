use crate::{db_init, repositories};
use sqlx::{Pool, Postgres};
use std::sync::Arc;

#[derive(Debug)]
pub struct AppState {
    pub db: Pool<Postgres>,
    pub repo: repositories::RepoImpls,
}

pub async fn create_app_state() -> Arc<AppState> {
    let pool = db_init::db_connect().await;
    let app_state = Arc::new(AppState {
        db: pool.clone(),
        repo: repositories::create_repositories().await,
    });
    app_state
}
