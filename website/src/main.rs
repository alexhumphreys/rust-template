//! Run with
//!
//! ```not_rust
//! cargo run -p example-templates
//! ```

use askama::Template;
use axum::{
    async_trait,
    body::{Body, Bytes},
    debug_handler, extract,
    extract::State,
    http::{Method, StatusCode},
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
use tracing::{self, info_span, instrument::WithSubscriber, Instrument, Span};
use tracing_opentelemetry::OpenTelemetrySpanExt;
use tracing_opentelemetry_instrumentation_sdk::find_current_trace_id;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: i32,
    pub anonymous: bool,
    pub username: String,
    pub permissions: HashSet<String>,
}

impl Default for User {
    fn default() -> Self {
        let mut permissions = HashSet::new();

        permissions.insert("Category::View".to_owned());

        Self {
            id: 1,
            anonymous: true,
            username: "Guest".into(),
            permissions,
        }
    }
}

// We place our Type within a Arc<> so we can send it across async threads.
type NullPool = Arc<Option<()>>;

#[async_trait]
impl Authentication<User, i64, NullPool> for User {
    async fn load_user(userid: i64, _pool: Option<&NullPool>) -> Result<User, anyhow::Error> {
        if userid == 1 {
            Ok(User::default())
        } else {
            let mut permissions = HashSet::new();

            permissions.insert("Category::View".to_owned());
            permissions.insert("Admin::View".to_owned());

            Ok(User {
                id: 2,
                anonymous: false,
                username: "Test".to_owned(),
                permissions,
            })
        }
    }

    fn is_authenticated(&self) -> bool {
        !self.anonymous
    }

    fn is_active(&self) -> bool {
        !self.anonymous
    }

    fn is_anonymous(&self) -> bool {
        self.anonymous
    }
}

#[async_trait]
impl HasPermission<NullPool> for User {
    async fn has(&self, perm: &str, _pool: &Option<&NullPool>) -> bool {
        self.permissions.contains(perm)
    }
}

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();

    init_subscribers_custom().ok();

    let session_config = SessionConfig::default().with_table_name("sessions_table");
    // create SessionStore and initiate the database tables
    let session_store = SessionStore::<SessionNullPool>::new(None, session_config)
        .await
        .unwrap();
    let auth_config = AuthConfig::<i64>::default();
    let nullpool = Arc::new(Option::None);

    // build our application with some routes
    let app = Router::new()
        .route("/greet/:name", get(greet))
        .route("/login", get(login).post(handle_login))
        .route("/login2", get(login2))
        .route("/perm", get(perm))
        .layer(
            AuthSessionLayer::<User, i64, SessionNullPool, NullPool>::new(Some(nullpool))
                .with_config(auth_config),
        )
        .layer(SessionLayer::new(session_store))
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

async fn login2(auth: AuthSession<User, i64, SessionNullPool, NullPool>) -> String {
    auth.login_user(2);
    "You are logged in as a User please try /perm to check permissions".to_owned()
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
async fn handle_login(extract::Form(input): extract::Form<Input>) -> Redirect {
    let login_payload = LoginPayload2 {
        name: input.name,
        password: input.password,
    };
    let user = match client::auth_user(login_payload, None).await {
        Ok(user) => user,
        Err(e) => {
            tracing::error!("TODO REMOVE error {:?}", e);
            return Redirect::to("/login");
        }
    };
    let template = HelloTemplate {
        name: format!("tried login for user: {:?}", user),
    };
    Redirect::to("/perm")
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

async fn perm(method: Method, auth: AuthSession<User, i64, SessionNullPool, NullPool>) -> String {
    let current_user = auth.current_user.clone().unwrap_or_default();

    //lets check permissions only and not worry about if they are anon or not
    if !Auth::<User, i64, NullPool>::build([Method::GET], false)
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
