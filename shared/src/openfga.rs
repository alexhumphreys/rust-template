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

pub fn get_client() -> ClientWithMiddleware {
    let client = ClientBuilder::new(reqwest::Client::new())
        // Trace HTTP requests. See the tracing crate to make use of these traces.
        .with_init(Extension(OtelName("openfga-client".into())))
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

#[tracing::instrument]
pub async fn get_client_by_token(
    token: String,
    headers: Option<HeaderMap>,
) -> Result<model::ClientModel, Error> {
    let http_client = get_client();
    let api_base_url = std::env::var("FGA_BASE_URL").expect("Define FGA_BASE_URL");

    let trace_headers = get_trace_info();
    let body = schema::ValidateToken { token };
    let req = http_client
        .get(format!("{}/api/clients/validate_token", api_base_url))
        .headers(headers.unwrap_or_default())
        .headers(trace_headers)
        .json::<schema::ValidateToken>(&body);

    tracing::info!("request being sent: {:?}", req);
    let res = req.send().await?;
    tracing::info!("response body: {:?}", res);
    todo!()
    /*
    match res.json::<DataBody<model::ClientModel>>().await {
        Ok(client) => {
            tracing::debug!("user id: {:?}", client.data.user_id);
            Ok(client.data)
        }
        Err(e) => {
            tracing::error!("Error fetching client: {:?}", e);
            Err(Error::Unauthorized)
        }
    }
    */
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CreateDataStoreSchema {
    pub name: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CreateDataStoreResponse {
    pub id: String,
    pub name: String,
    pub created_at: String,
    pub updated_at: String,
    // {"id":"01HJ3FP23AS376NVDBECMZBAPT", "name":"FGA Demo Store", "created_at":"2023-12-20T11:27:27.850720014Z", "updated_at":"2023-12-20T11:27:27.850720014Z"}%
}

#[tracing::instrument]
pub async fn create_data_store(
    store: CreateDataStoreSchema,
    headers: Option<HeaderMap>,
) -> Result<CreateDataStoreResponse, Error> {
    let http_client = get_client();
    //let fga_base_url = std::env::var("FGA_BASE_URL").expect("Define FGA_BASE_URL");
    let fga_base_url = "http://127.0.0.1:8080";
    let trace_headers = get_trace_info();

    let req = http_client
        .post(format!("{}/stores", fga_base_url))
        .headers(headers.unwrap_or_default())
        .headers(trace_headers)
        .json::<CreateDataStoreSchema>(&store);
    tracing::debug!("request being sent: {:?}", req);
    let res = req.send().await?;
    tracing::debug!("response body: {:?}", res);
    Ok(res.json::<CreateDataStoreResponse>().await?)
}

#[derive(Serialize, Deserialize, Debug)]
pub struct RelationshipTuple {
    user: String,
    relation: String,
    object: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum RelationshipAction {
    Write(RelationshipTuple),
    Delete(RelationshipTuple),
}
#[derive(Serialize, Deserialize, Debug)]
pub struct WriteRelationshipTupleSchema {
    pub authorization_model_id: String,
    pub action: RelationshipAction,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct WriteRelationshipTupleResponse {
    pub id: String,
    pub name: String,
    pub created_at: String,
    pub updated_at: String,
    // {"id":"01HJ3FP23AS376NVDBECMZBAPT", "name":"FGA Demo Store", "created_at":"2023-12-20T11:27:27.850720014Z", "updated_at":"2023-12-20T11:27:27.850720014Z"}%
}

#[tracing::instrument]
pub async fn write_relationship_tuple(
    store: CreateDataStoreSchema,
    headers: Option<HeaderMap>,
) -> Result<CreateDataStoreResponse, Error> {
    let http_client = get_client();
    //let fga_base_url = std::env::var("FGA_BASE_URL").expect("Define FGA_BASE_URL");
    let fga_base_url = "http://127.0.0.1:8080";
    let trace_headers = get_trace_info();

    let req = http_client
        .post(format!("{}/stores", fga_base_url))
        .headers(headers.unwrap_or_default())
        .headers(trace_headers)
        .json::<CreateDataStoreSchema>(&store);
    tracing::debug!("request being sent: {:?}", req);
    let res = req.send().await?;
    tracing::debug!("response body: {:?}", res);
    Ok(res.json::<CreateDataStoreResponse>().await?)
}

#[cfg(test)]
mod tests {
    use crate::openfga::*;

    #[tokio::test]
    async fn test_create_data_store() {
        let store_name = "foobar".to_string();
        let res = create_data_store(
            CreateDataStoreSchema {
                name: store_name.clone(),
            },
            None,
        )
        .await;

        assert_eq!(res.unwrap().name, store_name);
    }

    #[tokio::test]
    async fn test_json_serialize() {
        let json = WriteRelationshipTupleSchema {
            authorization_model_id: "123".to_string(),
            action: RelationshipAction::Write(RelationshipTuple {
                user: "user:456".to_string(),
                relation: "reader".to_string(),
                object: "document:z".to_string(),
            }),
        };
        println!("{:?}", serde_json::to_string(&json));
        assert_eq!(true, true);
    }
}
