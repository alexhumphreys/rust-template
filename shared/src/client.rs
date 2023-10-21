use crate::{
    error::Error, model, schema, telemetry::init_subscribers_custom,
    tracing::make_otel_reqwest_span,
};
use reqwest;
use reqwest::{
    header::USER_AGENT,
    header::{HeaderMap, HeaderName, HeaderValue},
};
use reqwest_middleware::{ClientBuilder, Extension};
use reqwest_tracing::{OtelName, TracingMiddleware};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use tracing::{self, info_span, Instrument, Span};

#[derive(Debug, Deserialize, Serialize)]
#[allow(non_snake_case)]
pub struct DataBody<T> {
    pub data: T,
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

pub async fn auth_user(
    payload: schema::LoginPayload2,
    headers: Option<HeaderMap>,
) -> Result<model::UserTransportModel, Error> {
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
        .send()
        .instrument(info_span!("some span"));
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
