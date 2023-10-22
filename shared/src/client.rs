use crate::{
    error::Error, model, schema, telemetry::init_subscribers_custom,
    tracing::make_otel_reqwest_span,
};
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
use reqwest_middleware::{ClientBuilder, ClientWithMiddleware, Extension};
use reqwest_tracing::{OtelName, ReqwestOtelSpanBackend, TracingMiddleware};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{self, info_span, instrument::WithSubscriber, subscriber, Instrument, Span};
use tracing_opentelemetry::OpenTelemetrySpanExt;
use tracing_opentelemetry_instrumentation_sdk::find_current_trace_id;

#[derive(Debug, Deserialize, Serialize)]
#[allow(non_snake_case)]
pub struct DataBody<T> {
    pub data: T,
}

fn get_client() -> ClientWithMiddleware {
    let client = ClientBuilder::new(reqwest::Client::new())
        // Trace HTTP requests. See the tracing crate to make use of these traces.
        .with_init(Extension(OtelName("my-client".into())))
        .with(TracingMiddleware::default())
        .build();

    client
}

fn get_trace_info() -> HeaderMap {
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
    tracing::debug!("traceId in get_trace_info:");
    tracing::debug!(trace_id);
    headers
}

pub async fn get_clients(headers: Option<HeaderMap>) {
    let api_base_url = std::env::var("API_BASE_URL").expect("Define API_BASE_URL");

    let client = ClientBuilder::new(reqwest::Client::new())
        // Trace HTTP requests. See the tracing crate to make use of these traces.
        .with_init(Extension(OtelName("my-client".into())))
        .with(TracingMiddleware::default())
        .build();
    let req = client
        .get(format!("{}/api/clients", api_base_url))
        .headers(headers.unwrap_or_default())
        .send()
        .instrument(info_span!("some span"));
    let res = req.await.unwrap();
    match res.json::<DataBody<model::AccountModel>>().await {
        Ok(json) => println!("{:#?}", json),
        Err(e) => println!("Error: {}", e),
    }
}

#[tracing::instrument]
pub async fn auth_user(
    payload: schema::LoginPayload2,
    headers: Option<HeaderMap>,
) -> Result<model::UserTransportModel, Error> {
    let trace_headers = get_trace_info();
    let api_base_url = std::env::var("API_BASE_URL").expect("Define API_BASE_URL");

    let client = ClientBuilder::new(reqwest::Client::new())
        // Trace HTTP requests. See the tracing crate to make use of these traces.
        .with_init(Extension(OtelName("my-client".into())))
        .with(TracingMiddleware::default())
        .build();

    let req = client
        .post(format!("{}/api/users/login", api_base_url))
        .json(&payload)
        .headers(headers.unwrap_or_default())
        .send();
    let res = req.await.unwrap();
    match res.json::<DataBody<model::UserTransportModel>>().await {
        Ok(user) => {
            println!("{:#?}", user);
            Ok(user.data)
        }
        Err(e) => {
            println!("Error: {}", e);
            Err(Error::Unauthorized)
        }
    }
}

#[tracing::instrument]
pub async fn get_user(
    id: uuid::Uuid,
    headers: Option<HeaderMap>,
) -> Result<model::UserTransportModel, Error> {
    let client = get_client();
    let api_base_url = std::env::var("API_BASE_URL").expect("Define API_BASE_URL");

    let req = client
        .get(format!("{}/api/users/{}", api_base_url, id))
        .headers(headers.unwrap_or_default())
        .send();
    let res = req.await.unwrap();
    match res.json::<DataBody<model::UserTransportModel>>().await {
        Ok(user) => {
            println!("{:#?}", user);
            Ok(user.data)
        }
        Err(e) => {
            println!("Error: {}", e);
            Err(Error::Unauthorized)
        }
    }
}
