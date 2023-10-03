use std::sync::Arc;

use crate::{db, schema::FilterOptions, AppState};
use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};

#[tracing::instrument]
pub async fn get_client_handler(
    opts: Option<Query<FilterOptions>>,
    State(data): State<Arc<AppState>>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    match db::get_client_list(opts, State(data)).await {
        Ok(clients) => {
            let json_response = serde_json::json!({
                "status": "success",
                "results": clients.len(),
                "clients": clients
            });
            Ok(Json(json_response))
        }
        Err(e) => {
            let error_response = serde_json::json!({
                "status": "fail",
                "message": e,
            });
            Err((StatusCode::INTERNAL_SERVER_ERROR, Json(error_response)))
        }
    }
}
