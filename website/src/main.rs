//! Run with
//!
//! ```not_rust
//! cargo run -p example-templates
//! ```
mod auth;
mod handlers;
mod protected_routes;

use askama::Template;
use axum::{
    async_trait,
    body::{Body, Bytes},
    debug_handler, extract,
    extract::State,
    http::{Method, StatusCode},
    middleware,
    response::{Html, IntoResponse, Json, Redirect, Response},
    routing::get,
    Router,
};
use axum_session::{Session, SessionConfig, SessionLayer, SessionNullPool, SessionStore};
use axum_session_auth::{
    Auth, AuthConfig, AuthSession, AuthSessionLayer, Authentication, HasPermission, Rights,
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
use serde::{Deserialize, Serialize};
use shared::{
    client, error::Error, model::UserTransportModel, schema::LoginPayload2,
    telemetry::init_subscribers_custom,
};
use std::sync::Arc;
use std::{collections::HashMap, collections::HashSet, net::SocketAddr};
use tower_layer::Layer;
use tracing::{self, info_span, instrument::WithSubscriber, Instrument, Span};
use tracing_opentelemetry::OpenTelemetrySpanExt;
use tracing_opentelemetry_instrumentation_sdk::find_current_trace_id;
use uuid::Uuid;

// We place our Type within a Arc<> so we can send it across async threads.
type NullPool = Arc<Option<()>>;
type AuthIdType = Option<Uuid>;
type AuthSessionType = AuthSession<auth::User, AuthIdType, SessionNullPool, NullPool>;
type AuthSessionLayerType = AuthSessionLayer<auth::User, AuthIdType, SessionNullPool, NullPool>;

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();

    init_subscribers_custom().ok();

    let public = Router::new()
        .route("/greet/:name", get(greet))
        .route("/login", get(login).post(handle_login))
        .route("/hit/api", get(proxy_via_reqwest))
        .route("/perm", get(perm));

    let (auth_session_layer, session_layer) = make_auth_session_layer().await;
    // build our application with some routes
    let app = Router::new()
        .merge(protected_routes::router())
        .merge(public)
        .layer(auth_session_layer)
        .layer(session_layer)
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

async fn make_auth_session_layer() -> (AuthSessionLayerType, SessionLayer<SessionNullPool>) {
    let session_config = SessionConfig::default().with_table_name("sessions_table");
    let session_store = SessionStore::<SessionNullPool>::new(None, session_config)
        .await
        .unwrap();
    let auth_config = AuthConfig::<Option<Uuid>>::default();
    let nullpool = Arc::new(Option::None);
    let layer = AuthSessionLayerType::new(Some(nullpool)).with_config(auth_config);
    (layer, SessionLayer::new(session_store))
}

async fn login() -> impl IntoResponse {
    let template = LoginTemplate {};
    HtmlTemplate(template)
}

#[derive(Template)]
#[template(path = "login.html")]
struct LoginTemplate {}

async fn greet(extract::Path(name): extract::Path<String>) -> impl IntoResponse {
    let template = HelloTemplate { name };
    HtmlTemplate(template)
}

#[derive(Deserialize, Debug)]
#[allow(dead_code)]
struct Input {
    name: String,
    password: String,
}

#[debug_handler]
async fn handle_login(
    auth: AuthSessionType,
    extract::Form(input): extract::Form<Input>,
) -> Redirect {
    let login_payload = LoginPayload2 {
        name: input.name,
        password: input.password,
    };
    match client::auth_user(login_payload, None).await {
        Ok(user) => {
            auth.login_user(Some(user.id));
            Redirect::to("/perm")
        }
        Err(e) => {
            tracing::error!("TODO REMOVE error {:?}", e);
            Redirect::to("/login")
        }
    }
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

async fn perm(method: Method, auth: AuthSessionType) -> String {
    let current_user = auth.current_user.clone().unwrap_or_default();

    //lets check permissions only and not worry about if they are anon or not
    if !Auth::<auth::User, Option<Uuid>, NullPool>::build([Method::GET], false)
        .requires(Rights::any([
            Rights::permission("Category::View"),
            Rights::permission("Admin::View"),
        ]))
        .validate(&current_user, &method, None)
        .await
    {
        return format!(
            "User {}, Does not have permissions needed to view this page please login",
            current_user.username
        );
    }

    format!(
        "User id {:?} and name {:?} has Permissions needed. Here are the Users permissions: {:?}",
        current_user.id, current_user.username, current_user.permissions
    )
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
