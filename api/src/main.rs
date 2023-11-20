mod app_state;
mod client_repository;
mod db;
mod db_init;
mod handler;
mod repositories;
mod router;
#[cfg(test)]
mod tests;
mod usecases;
mod user_repository;

use axum::{Json, Router};
use axum_otel_metrics::HttpMetricsLayerBuilder;
use axum_tracing_opentelemetry::middleware::{OtelAxumLayer, OtelInResponseLayer};
use dotenvy;
use oasgen::{openapi, OaSchema, Server};
use serde::{Deserialize, Serialize};
use shared;
use shared::telemetry::init_subscribers_custom;
use std::net::SocketAddr;

#[derive(OaSchema, Deserialize)]
pub struct SendCode {
    pub mobile: String,
}

#[derive(Serialize, OaSchema, Debug)]
pub struct SendCodeResponse {
    pub found_account: bool,
}

#[openapi]
async fn send_code(_body: Json<SendCode>) -> Json<SendCodeResponse> {
    Json(SendCodeResponse {
        found_account: false,
    })
}

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();

    init_subscribers_custom().ok();

    let metrics = HttpMetricsLayerBuilder::new().build();

    let server = Server::axum()
        .post("/send-code", send_code)
        .write_and_exit_if_env_var_set("openapi.yaml") // set OASGEN_WRITE_SPEC=1
        .freeze();

    let app = Router::new()
        .merge(metrics.routes()) // TODO other port?
        .merge(router::routes().await)
        // include trace context as header into the response
        .layer(OtelInResponseLayer::default())
        //start OpenTelemetry trace on incoming request
        .layer(OtelAxumLayer::default())
        .layer(metrics)
        .merge(server.into_router());

    // run it
    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    tracing::info!("listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}
