use crate::{
    error::Error,
    model::{AccountModel, ClientModel},
    schema::FilterOptions,
    AppState,
};
use anyhow::Result;
use axum::extract::{Query, State};
use axum::http::StatusCode;
use shared::tracing::make_otel_db_span;
use sqlx::Execute;
use std::sync::Arc;
use tracing::{self, Instrument};
use uuid::Uuid;

pub async fn get_client_list(
    opts: Option<Query<FilterOptions>>,
    State(data): State<Arc<AppState>>,
) -> Result<Vec<ClientModel>, Error> {
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
        .await?;

    Ok(query_result)
}

async fn get_account(
    account_id: Uuid,
    State(data): State<Arc<AppState>>,
) -> Result<AccountModel, Error> {
    let query = sqlx::query_as!(
        AccountModel,
        "SELECT * FROM accounts WHERE id = $1",
        account_id
    );
    let sql = query.sql().clone();
    let query_result = query
        .fetch_one(&data.db)
        .instrument(make_otel_db_span("SELECT", sql))
        .await?;

    Ok(query_result)
}
