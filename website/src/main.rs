//! Run with
//!
//! ```not_rust
//! cargo run -p example-templates
//! ```

use askama::Template;
use axum::{
    body::{Body, Bytes},
    extract,
    extract::State,
    http::StatusCode,
    response::{Html, IntoResponse, Json, Response},
    routing::get,
    Router,
};
use axum_tracing_opentelemetry::middleware::{OtelAxumLayer, OtelInResponseLayer};
use dotenvy;
use opentelemetry::{
    global,
    global::set_text_map_propagator,
    propagation::TextMapPropagator,
    sdk::propagation::TraceContextPropagator,
    trace::{TraceContextExt, Tracer},
};
use reqwest;
use reqwest::{
    header::USER_AGENT,
    header::{HeaderMap, HeaderName, HeaderValue},
};
use reqwest_middleware::{ClientBuilder, Extension};
use reqwest_tracing::{OtelName, TracingMiddleware};
use shared::{error::Error, telemetry::init_subscribers_custom, tracing::make_otel_reqwest_span};
use std::{collections::HashMap, net::SocketAddr};
use tracing::{self, info_span, instrument::WithSubscriber, Instrument, Span};
use tracing_opentelemetry::OpenTelemetrySpanExt;
use tracing_opentelemetry_instrumentation_sdk::find_current_trace_id;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();

    init_subscribers_custom().ok();

    // build our application with some routes
    let app = Router::new()
        .route("/greet/:name", get(greet))
        .route("/hit/api", get(proxy_via_reqwest))
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

async fn greet(extract::Path(name): extract::Path<String>) -> impl IntoResponse {
    let template = HelloTemplate { name };
    HtmlTemplate(template)
}

#[derive(Template)]
#[template(path = "hello.html")]
struct HelloTemplate {
    name: String,
}

struct HtmlTemplate<T>(T);

impl<T> IntoResponse for HtmlTemplate<T>
where
    T: Template,
{
    fn into_response(self) -> Response {
        match self.0.render() {
            Ok(html) => Html(html).into_response(),
            Err(err) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to render template. Error: {}", err),
            )
                .into_response(),
        }
    }
}

#[tracing::instrument]
async fn proxy_via_reqwest() -> Result<impl IntoResponse, Error> {
    let span = tracing::Span::current();
    let context = span.context();
    let propagator = TraceContextPropagator::new();
    let mut fields = HashMap::new();
    propagator.inject_context(&context, &mut fields);
    propagator.inject_context(&context, &mut fields);
    let headers = fields
        .into_iter()
        .map(|(k, v)| {
            (
                HeaderName::try_from(k).unwrap(),
                HeaderValue::try_from(v).unwrap(),
            )
        })
        .collect();

    set_text_map_propagator(TraceContextPropagator::new());
    let trace_id = find_current_trace_id().unwrap_or("".to_string());
    tracing::info!("traceId:");
    tracing::info!(trace_id);
    let api_base_url = std::env::var("API_BASE_URL").expect("Define API_BASE_URL");

    let client = ClientBuilder::new(reqwest::Client::new())
        // Trace HTTP requests. See the tracing crate to make use of these traces.
        .with_init(Extension(OtelName("my-client".into())))
        .with(TracingMiddleware::default())
        .build();
    let req = client
        .get(format!("{}/api/clients", api_base_url))
        .headers(headers)
        .send()
        .instrument(info_span!("some span"));
    let res = req.await.unwrap();

    let text = res.text().await?;
    Ok(Json(text).into_response())
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
