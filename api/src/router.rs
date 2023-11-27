use crate::{app_state::create_app_state, handler};
use aide::{
    axum::{
        routing::{get, post, put},
        ApiRouter, IntoApiResponse,
    },
    openapi::{Info, OpenApi},
};
use axum::Router;
use axum::{Extension, Json};

async fn serve_api(Extension(api): Extension<OpenApi>) -> impl IntoApiResponse {
    Json(api)
}

pub async fn routes() -> ApiRouter {
    let app_state = create_app_state().await;

    let router = ApiRouter::new()
        .api_route("/", get(handler::handler))
        .api_route("/api/healthz", get(handler::health_checker_handler))
        //.route_layer(middleware::from_fn(auth::auth))
        .api_route(
            "/api/clients/validate_token",
            get(handler::get_client_by_token),
        )
        .api_route("/api/clients", get(handler::get_client_handler))
        .api_route("/api/clients", post(handler::create_client))
        .api_route("/api/accounts/:id", put(handler::put_account))
        .api_route("/api/accounts/:id", get(handler::get_account))
        .api_route("/api/accounts", get(handler::get_account))
        .api_route("/api/accounts", post(handler::create_account))
        .api_route("/api/users/login", post(handler::validate_user))
        .api_route("/api/users/:id", get(handler::get_user))
        .api_route("/api/users", post(handler::create_user))
        .with_state(app_state);
    router
}
