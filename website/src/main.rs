//! Run with
//!
//! ```not_rust
//! cargo run -p example-templates
//! ```
mod auth;
mod handlers;
mod protected_routes;
mod proxy_routes;
mod public_routes;

use axum::Router;
use axum_tracing_opentelemetry::middleware::{OtelAxumLayer, OtelInResponseLayer};
use dotenvy;
use shared::telemetry::init_subscribers_custom;
use std::net::SocketAddr;

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();

    init_subscribers_custom().ok();

    let (auth_session_layer, session_layer) = auth::make_auth_session_layer().await;

    let app = Router::new()
        .merge(protected_routes::router())
        .merge(public_routes::router())
        // include authentication session middleware
        .layer(auth_session_layer)
        // include session storage
        .layer(session_layer)
        // include proxy after the session auth
        .merge(proxy_routes::router())
        // include trace context as header into the response
        .layer(OtelInResponseLayer::default())
        //start OpenTelemetry trace on incoming request
        .layer(OtelAxumLayer::default());

    // run it
    let addr = SocketAddr::from(([0, 0, 0, 0], 8000));
    tracing::info!("listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
