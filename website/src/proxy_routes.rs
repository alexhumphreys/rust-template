use crate::{auth, handlers};
use axum::{
    debug_handler,
    extract::{Path, Query, State},
    http::Method,
    response::IntoResponse,
    routing::get,
    Json, Router,
};
use hyper::StatusCode;
use reqwest::Response;
use reqwest_middleware::ClientWithMiddleware;
use serde_json::Value;
use shared::{client, error::Error};
use std::collections::HashMap;
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct AppState {
    client: ClientWithMiddleware,
}

pub fn router() -> Router {
    let client = client::get_client();
    let app_state = (AppState {
        client: client.clone(),
    });

    let proxy = Router::new()
        .route("/api/:path", get(proxy_handler))
        .with_state(app_state);
    // TODO proxy auth
    // .route_layer(middleware::from_fn(auth::session_auth));
    proxy
}

// Proxy Routes

#[debug_handler]
async fn proxy_handler(
    method: Method,
    query: Query<HashMap<String, String>>,
    path: Path<String>,
    State(client): State<AppState>,
    body: String,
) -> Result<impl IntoResponse, axum::http::StatusCode> {
    let api_base_url = std::env::var("API_BASE_URL").expect("Define API_BASE_URL");
    let url = format!("{}/api/{}", api_base_url, path.to_string());

    let req = (client.client)
        .request(method, url)
        .send()
        .await
        .map_err(|e| {
            tracing::error!("TODO REMOVE error {:?}", e);
            axum::http::StatusCode::INTERNAL_SERVER_ERROR
        })?;

    tracing::info!("Req output {:?}", req);

    let status = req.status().clone();
    let json = req.json::<Value>().await.map_err(|e| {
        tracing::error!("TODO REMOVE error {:?}", e);
        axum::http::StatusCode::INTERNAL_SERVER_ERROR
    })?;

    // TODO proper status
    Ok(Json(json))
}
