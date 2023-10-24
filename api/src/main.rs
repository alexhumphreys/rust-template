mod db;
mod db_init;
mod handler;
mod repository;
mod user_repository;

use axum::{
    middleware,
    response::{Html, IntoResponse},
    routing::{get, post, put},
    Json, Router,
};
use axum_otel_metrics::HttpMetricsLayerBuilder;
use axum_tracing_opentelemetry::middleware::{OtelAxumLayer, OtelInResponseLayer};
use dotenvy;
use oasgen::{openapi, OaSchema, Server};
use serde::{Deserialize, Serialize};
use shared;
use shared::{auth, telemetry::init_subscribers_custom};
use sqlx::{postgres::PgPoolOptions, Pool, Postgres};
use std::net::SocketAddr;
use std::sync::Arc;

#[derive(Debug)]
pub struct AppState {
    db: Pool<Postgres>,
    repo: repository::RepoImpls,
}

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

    //init_tracing_opentelemetry::tracing_subscriber_ext::init_subscribers().ok();
    init_subscribers_custom().ok();

    let metrics = HttpMetricsLayerBuilder::new().build();

    let pool = db_init::db_connect().await;
    let app_state = Arc::new(AppState {
        db: pool.clone(),
        repo: repository::create_repositories().await,
    });

    let server = Server::axum()
        .post("/send-code", send_code)
        .write_and_exit_if_env_var_set("openapi.yaml") // set OASGEN_WRITE_SPEC=1
        .freeze();

    let app = Router::new()
        .merge(metrics.routes()) // TODO other port?
        .route("/", get(handler))
        .route("/api/healthz", get(health_checker_handler))
        //.route_layer(middleware::from_fn(auth::auth))
        .route("/404", get(four_handler))
        .route("/api/clients", get(handler::get_client_handler))
        .route("/api/accounts/:id", put(handler::put_account))
        .route("/api/accounts/:id", get(handler::get_account))
        .route("/api/accounts", get(handler::get_account))
        .route("/api/accounts", post(handler::create_account))
        .route("/api/users/login", post(handler::validate_user))
        .route("/api/users/:id", get(handler::get_user))
        .route("/api/users", post(handler::create_user))
        .with_state(app_state)
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

async fn health_checker_handler() -> impl IntoResponse {
    const MESSAGE: &str = "Simple CRUD API with Rust, SQLX, Postgres,and Axum";

    let json_response = serde_json::json!({
        "status": "success",
        "message": MESSAGE
    });

    Json(json_response)
}

async fn handler() -> Html<&'static str> {
    Html("<h1>Hello, World!</h1>")
}

async fn four_handler() -> impl IntoResponse {
    shared::error::Error::NotFound
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
