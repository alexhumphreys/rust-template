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
use serde_json::Value;
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
    let fga_base_url = std::env::var("FGA_BASE_URL").expect("Define FGA_BASE_URL");
    // let fga_base_url = "http://127.0.0.1:8080";
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

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RelationshipTuple {
    pub user: String,
    pub relation: String,
    pub object: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct TupleKeys {
    pub tuple_keys: Vec<RelationshipTuple>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "lowercase")]
pub enum RelationshipAction {
    Writes(TupleKeys),
    Deletes(TupleKeys),
}
#[derive(Serialize, Deserialize, Debug)]
pub struct WriteRelationshipTupleSchema {
    pub authorization_model_id: String,

    #[serde(flatten)]
    pub relationship_action: RelationshipAction,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct WriteRelationshipTupleResponse {
    // {}%
}

#[tracing::instrument]
pub async fn write_relationship_tuple(
    store_id: String,
    tuples: WriteRelationshipTupleSchema,
    headers: Option<HeaderMap>,
) -> Result<WriteRelationshipTupleResponse, Error> {
    let http_client = get_client();
    //let fga_base_url = std::env::var("FGA_BASE_URL").expect("Define FGA_BASE_URL");
    let fga_base_url = "http://127.0.0.1:8080";
    let trace_headers = get_trace_info();

    let req = http_client
        .post(format!("{}/stores/{}/write", fga_base_url, store_id))
        .headers(headers.unwrap_or_default())
        .headers(trace_headers)
        .json::<WriteRelationshipTupleSchema>(&tuples);
    tracing::debug!("request being sent: {:?}", req);
    let res = req.send().await?;
    tracing::debug!("response body: {:?}", res);
    println!("{:?}", res);
    Ok(res.json::<WriteRelationshipTupleResponse>().await?)
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct WriteAuthorizationModelResponse {
    // {"authorization_model_id":"01HJ8QTTREQ7QJ9P5BCK0RHH5F"}
    pub authorization_model_id: String,
}

#[tracing::instrument]
pub async fn write_authorization_model(
    store_id: String,
    model: String,
    headers: Option<HeaderMap>,
) -> Result<WriteAuthorizationModelResponse, Error> {
    let http_client = get_client();
    //let fga_base_url = std::env::var("FGA_BASE_URL").expect("Define FGA_BASE_URL");
    let fga_base_url = "http://127.0.0.1:8080";
    let trace_headers = get_trace_info();

    let v: Value = serde_json::from_str(&model)?;

    let req = http_client
        .post(format!(
            "{}/stores/{}/authorization-models",
            fga_base_url, store_id
        ))
        .headers(headers.unwrap_or_default())
        .headers(trace_headers)
        .json::<Value>(&v);
    tracing::debug!("request being sent: {:?}", req);
    let res = req.send().await?;
    tracing::debug!("response body: {:?}", res);
    println!("{:?}", res);
    Ok(res.json::<WriteAuthorizationModelResponse>().await?)
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CheckRequest {
    // {"allowed":true}
    pub authorization_model_id: String,
    pub tuple_key: RelationshipTuple,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct CheckResponse {
    // {"allowed":true}
    pub allowed: bool,
}

#[tracing::instrument]
pub async fn check(
    store_id: String,
    model_id: String,
    tuple: RelationshipTuple,
    headers: Option<HeaderMap>,
) -> Result<CheckResponse, Error> {
    let http_client = get_client();
    //let fga_base_url = std::env::var("FGA_BASE_URL").expect("Define FGA_BASE_URL");
    let fga_base_url = "http://127.0.0.1:8080";
    let trace_headers = get_trace_info();

    let body = CheckRequest {
        authorization_model_id: model_id,
        tuple_key: tuple,
    };
    let req = http_client
        .post(format!("{}/stores/{}/check", fga_base_url, store_id))
        .headers(headers.unwrap_or_default())
        .headers(trace_headers)
        .json::<CheckRequest>(&body);
    tracing::debug!("request being sent: {:?}", req);
    let res = req.send().await?;
    tracing::debug!("response body: {:?}", res);
    println!("{:?}", res);
    Ok(res.json::<CheckResponse>().await?)
}

pub fn make_tuple(user: &str, relation: &str, object: &str) -> RelationshipTuple {
    RelationshipTuple {
        user: user.to_string(),
        relation: relation.to_string(),
        object: object.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use crate::openfga::*;

    #[tokio::test]
    async fn test_openfga_create_data_store() {
        let store_name = "foobar2".to_string();
        let res = create_data_store(
            CreateDataStoreSchema {
                name: store_name.clone(),
            },
            None,
        )
        .await;

        let store = res.unwrap();
        assert_eq!(store.name, store_name);

        let model_string = r#"
        {"schema_version":"1.1","type_definitions":[{"type":"user"},{"type":"document","relations":{"reader":{"this":{}},"writer":{"this":{}},"owner":{"this":{}}},"metadata":{"relations":{"reader":{"directly_related_user_types":[{"type":"user"}]},"writer":{"directly_related_user_types":[{"type":"user"}]},"owner":{"directly_related_user_types":[{"type":"user"}]}}}}]}
        "#.to_string();
        let model = write_authorization_model(store.id.clone(), model_string, None).await;
        let authorization_model_id = model.unwrap().authorization_model_id;
        let tuple = make_tuple("user:789", "reader", "document:z");
        let json = WriteRelationshipTupleSchema {
            authorization_model_id: authorization_model_id.clone(),
            relationship_action: RelationshipAction::Writes(TupleKeys {
                tuple_keys: vec![tuple.clone()],
            }),
        };
        println!("{:?}", serde_json::to_string(&json));
        let res = write_relationship_tuple(store.id.clone(), json, None).await;
        assert_eq!(res.unwrap(), WriteRelationshipTupleResponse {});
        let allowed = check(
            store.id.clone(),
            authorization_model_id.clone(),
            tuple,
            None,
        )
        .await;
        assert_eq!(allowed.unwrap().allowed, true);
    }

    #[tokio::test]
    async fn test_openfga_json_serialize() {
        let json = WriteRelationshipTupleSchema {
            authorization_model_id: "123".to_string(),
            relationship_action: RelationshipAction::Writes(TupleKeys {
                tuple_keys: vec![RelationshipTuple {
                    user: "user:789".to_string(),
                    relation: "reader".to_string(),
                    object: "document:z".to_string(),
                }],
            }),
        };
        println!("{:?}", serde_json::to_string(&json));
        assert_eq!(true, true);
    }
}
