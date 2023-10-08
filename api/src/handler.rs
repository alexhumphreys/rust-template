use crate::{db, error::Error, schema::FilterOptions, AppState};
use axum::{
    extract::{Query, State},
    response::IntoResponse,
    Json,
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
        "clients": clients
    });
    Ok(Json(json_response))
}
