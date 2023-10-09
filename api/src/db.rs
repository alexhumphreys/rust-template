use crate::{
    model::{AccountModel, ClientModel},
    schema, AppState,
};
use anyhow::{Context, Result};
use axum::extract::{Query, State};
use axum::http::StatusCode;
use shared::{error::Error, tracing::make_otel_db_span};
use sqlx::Execute;
use std::sync::Arc;
use tracing::{self, Instrument};
use uuid::Uuid;

pub async fn get_client_list(
    opts: Option<Query<schema::FilterOptions>>,
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

pub async fn get_account(
    account_id: Uuid,
    State(data): State<Arc<AppState>>,
) -> Result<AccountModel, Error> {
    tracing::debug!("Searching for account id {}", account_id.to_string());
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
    //.with_context(|| format!("No value found for: {}", account_id.to_string()))?;

    Ok(query_result)
}

pub async fn create_account(
    account: schema::CreateAccount,
    State(data): State<Arc<AppState>>,
) -> Result<AccountModel, Error> {
    tracing::debug!("creating account");
    let query = sqlx::query_as!(
        AccountModel,
        "INSERT INTO accounts(name, credential) VALUES ($1, $2) RETURNING *",
        account.name,
        account.credential
    );
    let sql = query.sql().clone();
    let query_result = query
        .fetch_one(&data.db)
        .instrument(make_otel_db_span("INSERT", sql))
        .await?;

    Ok(query_result)
}

pub async fn put_account(
    id: Uuid,
    account: schema::CreateAccount,
    State(data): State<Arc<AppState>>,
) -> Result<AccountModel, Error> {
    tracing::debug!("creating account");
    let query = sqlx::query_as!(
        AccountModel,
        "UPDATE accounts SET name = $1, credential = $2 WHERE id = $3 RETURNING *",
        account.name,
        account.credential,
        id
    );
    let sql = query.sql().clone();
    let query_result = query
        .fetch_one(&data.db)
        .instrument(make_otel_db_span("INSERT", sql))
        .await?;

    Ok(query_result)
}
