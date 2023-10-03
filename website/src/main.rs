//! Run with
//!
//! ```not_rust
//! cargo run -p example-templates
//! ```

use api_client::{self, apis::configuration};
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
use dotenvy;
use reqwest;
use std::net::SocketAddr;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();

    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "example_templates=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // build our application with some routes
    let app = Router::new()
        .route("/greet/:name", get(greet))
        .route("/hit/client", get(via_lib))
        .route("/hit/api", get(proxy_via_reqwest));

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

async fn proxy_via_reqwest() -> impl IntoResponse {
    let api_base_url = std::env::var("API_BASE_URL").expect("Define API_BASE_URL");
    let reqwest_response = match reqwest::get(format!("{}/api/clients", api_base_url)).await {
        Ok(res) => res,
        Err(err) => {
            tracing::error!(%err, "request failed");
            return (axum::http::StatusCode::INTERNAL_SERVER_ERROR, "foo").into_response();
        }
    };

    Json(reqwest_response.text().await.unwrap()).into_response()
}

async fn via_lib() -> impl IntoResponse {
    let api_base_url = std::env::var("API_BASE_URL").expect("Define API_BASE_URL");
    let configuration = configuration::Configuration {
        base_path: api_base_url,
        ..Default::default()
    };
    let arg = api_client::models::SendCode::new("1234".to_string());

    let res = api_client::apis::default_api::send_code(&configuration, arg)
        .await
        .ok();
    Json(res).into_response()
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
