use crate::{auth, handlers};
use axum::{
    debug_handler,
    extract::{Path, Query, RawQuery, State},
    http::Method,
    response::IntoResponse,
    routing::get,
    Json, Router,
};
use hyper::StatusCode;
use reqwest::Response;
use reqwest::Url;
use reqwest_middleware::ClientWithMiddleware;
use serde::{Deserialize, Serialize};
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
        .route(
            "/api/:path",
            get(proxy_handler)
                .post(proxy_handler)
                .delete(proxy_handler)
                .put(proxy_handler),
        )
        .with_state(app_state);
    // TODO proxy auth
    // .route_layer(middleware::from_fn(auth::session_auth));
    proxy
}

// Proxy Routes

#[debug_handler]
async fn proxy_handler(
    method: Method,
    query: RawQuery,
    path: Path<String>,
    State(client): State<AppState>,
    body_: Option<Json<Value>>,
) -> Result<impl IntoResponse, Error> {
    let api_base_url = std::env::var("API_BASE_URL").expect("Define API_BASE_URL");
    tracing::debug!("Request method {:?}", method);
    tracing::debug!("Query params {:?}", query);

    let host_path = format!(
        "{}/api/{}?{}",
        api_base_url,
        path.to_string(),
        query.0.unwrap_or_default()
    );
    let url = Url::parse(&host_path).map_err(|e| {
        tracing::error!("Error parsing url: {:?}", e);
        Error::BadRequest
    })?;

    let req = match body_ {
        Some(body) => {
            client
                .client
                .request(method, url.clone())
                .json::<Value>(&body)
                .send()
                .await?
        }
        None => client.client.request(method, url.clone()).send().await?,
    };

    tracing::info!("Req output {:?}", req);

    let status = req.status().clone();
    let json = req.json::<Value>().await?;

    // TODO proper status
    Ok(Json(json))
}
