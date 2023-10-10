use crate::{db, AppState};
use axum::{debug_handler, response::IntoResponse};
use axum::{
    extract::{Path, Query, State},
    Json,
};
use shared::schema::PathName;
use shared::{
    error::Error,
    schema::{CreateAccount, FilterOptions, PathId},
};
use std::sync::Arc;

#[tracing::instrument]
pub async fn get_client_handler(
    opts: Option<Query<FilterOptions>>,
    State(data): State<Arc<AppState>>,
) -> Result<impl IntoResponse, Error> {
    let clients = db::get_client_list(opts, State(data)).await?;
    let json_response = serde_json::json!({
        "status": "success",
        "results": clients.len(),
        "data": clients
    });
    Ok(Json(json_response))
}

#[tracing::instrument]
pub async fn get_account(
    Path(id): Path<PathId>,
    State(data): State<Arc<AppState>>,
) -> Result<impl IntoResponse, Error> {
    let account = db::get_account(id.id, State(data)).await?;
    let json_response = serde_json::json!({
        "status": "success",
        "results": 1,
        "data": account
    });
    Ok(Json(json_response))
}

#[tracing::instrument]
pub async fn search_account(
    Query(name): Query<PathName>,
    State(data): State<Arc<AppState>>,
) -> Result<impl IntoResponse, Error> {
    let account = db::get_account_by_name(name.name, State(data)).await?;
    let json_response = serde_json::json!({
        "status": "success",
        "results": 1,
        "data": account
    });
    Ok(Json(json_response))
}

#[tracing::instrument]
pub async fn create_account(
    State(data): State<Arc<AppState>>,
    Json(payload): Json<CreateAccount>,
) -> Result<impl IntoResponse, Error> {
    let account = db::create_account(payload, State(data)).await?;
    let json_response = serde_json::json!({
        "data": account
    });
    Ok(Json(json_response))
}

#[debug_handler]
#[tracing::instrument]
pub async fn put_account(
    Path(id): Path<PathId>,
    State(data): State<Arc<AppState>>,
    Json(payload): Json<CreateAccount>,
) -> Result<impl IntoResponse, Error> {
    let account = db::put_account(id.id, payload, State(data)).await?;
    let json_response = serde_json::json!({
    "data": account
    });
    Ok(Json(json_response))
}
