use crate::client_repository::ClientRepo;
use crate::repositories::Repositories;
use crate::user_repository::UserRepo;
use crate::{db, AppState};
use axum::{debug_handler, response::IntoResponse};
use axum::{
    extract::{Path, Query, State},
    Json,
};
use serde::Serialize;
use shared::schema::{CreateClient, LoginPayload, PathName};
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

//#[debug_handler]
#[tracing::instrument]
pub async fn put_account(
    Path(id): Path<PathId>,
    State(data): State<Arc<AppState>>,
    Json(payload): Json<CreateAccount>,
) -> Result<impl IntoResponse, Error> {
    let account = db::put_account(id.id, payload, State(data)).await?;
    Ok(wrap_response(account))
}

// User routes

#[tracing::instrument]
pub async fn create_user(
    State(data): State<Arc<AppState>>,
    Json(payload): Json<LoginPayload>,
) -> Result<impl IntoResponse, Error> {
    let user = data.repo.user().create_user(payload).await?;
    Ok(wrap_response(user))
}

#[debug_handler]
#[tracing::instrument]
pub async fn validate_user(
    State(data): State<Arc<AppState>>,
    Json(payload): Json<LoginPayload>,
) -> Result<impl IntoResponse, Error> {
    let user = data.repo.user().validate_credentials(payload).await?;
    Ok(wrap_response(user))
}

pub async fn get_user(
    Path(id): Path<PathId>,
    State(data): State<Arc<AppState>>,
) -> Result<impl IntoResponse, Error> {
    let user = data.repo.user().get_user(id.id).await?;
    Ok(wrap_response(user))
}

// Client routes

#[tracing::instrument]
pub async fn create_client(
    State(data): State<Arc<AppState>>,
    Json(payload): Json<CreateClient>,
) -> Result<impl IntoResponse, Error> {
    let client = data
        .repo
        .client()
        .create_client(payload.user_id, payload.name)
        .await?;
    Ok(wrap_response(client))
}

fn wrap_response(data: impl Serialize) -> impl IntoResponse {
    let json_response = serde_json::json!({
        "data": data
    });
    Json(json_response)
}
