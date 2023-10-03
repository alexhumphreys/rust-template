mod db;
mod handler;
mod model;
mod schema;

use api_server::init_subscribers_custom;
use axum::{
    response::{Html, IntoResponse},
    routing::get,
    Json, Router,
};
use axum_otel_metrics::HttpMetricsLayerBuilder;
use axum_tracing_opentelemetry::middleware::{OtelAxumLayer, OtelInResponseLayer};
use dotenvy;
use oasgen::{openapi, OaSchema, Server};
use serde::{Deserialize, Serialize};
use sqlx::{postgres::PgPoolOptions, Pool, Postgres};
use std::net::SocketAddr;
use std::sync::Arc;

#[derive(Debug)]
pub struct AppState {
    db: Pool<Postgres>,
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

    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let pool = match PgPoolOptions::new()
        .max_connections(10)
        .connect(&database_url)
        .await
    {
        Ok(pool) => {
            tracing::info!("✅Connection to the database is successful!");
            pool
        }
        Err(err) => {
            tracing::error!("🔥 Failed to connect to the database: {:?}", err);
            std::process::exit(1);
        }
    };

    let app_state = Arc::new(AppState { db: pool.clone() });

    let server = Server::axum()
        .post("/send-code", send_code)
        .write_and_exit_if_env_var_set("openapi.yaml") // set OASGEN_WRITE_SPEC=1
        .freeze();

    let app = Router::new()
        .merge(metrics.routes()) // TODO other port?
        .route("/", get(handler))
        .route("/api/healthz", get(health_checker_handler))
        .route("/api/clients", get(handler::get_client_handler))
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

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
