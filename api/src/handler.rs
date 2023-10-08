use crate::{
    db,
    error::Error,
    schema::{FilterOptions, ParamOptions, PathId},
    AppState,
};
use axum::{
    extract::{Path, Query, State},
    response::IntoResponse,
    Json,
};
use std::sync::Arc;
use uuid::Uuid;

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
