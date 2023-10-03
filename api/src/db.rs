use crate::{db, model::ClientModel, schema::FilterOptions, AppState};
use axum::extract::{Query, State};
use shared::tracing::make_otel_db_span;
use sqlx::Execute;
use std::sync::Arc;
use tracing::{self, Instrument};

pub async fn get_client_list(
    opts: Option<Query<FilterOptions>>,
    State(data): State<Arc<AppState>>,
) -> Result<Vec<ClientModel>, String> {
    let Query(opts) = opts.unwrap_or_default();

    let limit = opts.limit.unwrap_or(10);
    let offset = (opts.page.unwrap_or(1) - 1) * limit;

    let query = sqlx::query_as!(
        ClientModel,
        "SELECT * FROM clients ORDER by id LIMIT $1 OFFSET $2",
        limit as i32,
        offset as i32
    );

    let sql = query.sql().clone();
    let query_result = query
        .fetch_all(&data.db)
        .instrument(make_otel_db_span("SELECT", sql))
        .await;

    match query_result {
        Ok(clients) => return Ok(clients),
        Err(e) => {
            let msg = "Something bad happened while fetching all client items";
            tracing::error!("{}: {}", msg, e);
            Err(msg.to_string())
        }
    }
}
