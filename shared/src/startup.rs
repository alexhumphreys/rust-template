use crate::telemetry::init_subscribers_custom;
use axum::Router;
use axum_otel_metrics::HttpMetricsLayerBuilder;
use axum_tracing_opentelemetry::middleware::{OtelAxumLayer, OtelInResponseLayer};
use dotenvy;

pub fn server_setup() {
    dotenvy::dotenv().ok();

    init_subscribers_custom().ok();
}

pub fn create_server(router: Vec<Router>) -> Router {
    let routes = router
        .iter()
        .fold(Router::new(), |acc, r| acc.merge(r.to_owned()));

    let metrics = HttpMetricsLayerBuilder::new().build();
    let with_metrics = routes.merge(metrics.routes());
    let app = with_metrics
        // include trace context as header into the response
        .layer(OtelInResponseLayer::default())
        //start OpenTelemetry trace on incoming request
        .layer(OtelAxumLayer::default())
        .layer(metrics);
    app
}
