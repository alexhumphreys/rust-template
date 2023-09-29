//! Run with
//!
//! ```not_rust
//! cargo run -p example-hello-world
//! ```

use axum::{response::Html, routing::get, Router};
use axum_otel_metrics::HttpMetricsLayerBuilder;
use dotenvy;
use std::net::SocketAddr;

use tower_http::{
    self,
    classify::{ServerErrorsAsFailures, SharedClassifier},
    trace,
    trace::TraceLayer,
};
use tracing::Level;

fn setup_logging() {
    let logging_format = std::env::var("LOGGING_FMT").unwrap_or("json".to_owned());

    match logging_format.as_str() {
        "json" => {
            tracing_subscriber::fmt().with_target(false).json().init();
        }
        "pretty" => {
            tracing_subscriber::fmt().with_target(false).pretty().init();
        }
        _ => {
            tracing_subscriber::fmt()
                .with_target(false)
                .compact()
                .init();
        }
    }
}

fn create_trace_layer(
) -> TraceLayer<SharedClassifier<ServerErrorsAsFailures>, trace::DefaultMakeSpan> {
    TraceLayer::new_for_http()
        .make_span_with(trace::DefaultMakeSpan::new().level(Level::INFO))
        .on_response(trace::DefaultOnResponse::new().level(Level::INFO))
}

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();

    setup_logging();

    let metrics = HttpMetricsLayerBuilder::new().build();

    // build our application with a route
    let app = Router::new()
        .merge(metrics.routes()) // TODO other port?
        .route("/", get(handler))
        .layer(create_trace_layer())
        .layer(metrics);

    // run it
    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    println!("listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
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
