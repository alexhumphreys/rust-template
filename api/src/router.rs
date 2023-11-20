use crate::{app_state::create_app_state, handler};
use axum::{
    routing::{get, post, put},
    Router,
};

pub async fn routes() -> Router {
    let app_state = create_app_state().await;

    let router = Router::new()
        .route("/", get(handler::handler))
        .route("/api/healthz", get(handler::health_checker_handler))
        //.route_layer(middleware::from_fn(auth::auth))
        .route(
            "/api/clients/validate_token",
            get(handler::get_client_by_token),
        )
        .route("/api/clients", get(handler::get_client_handler))
        .route("/api/clients", post(handler::create_client))
        .route("/api/accounts/:id", put(handler::put_account))
        .route("/api/accounts/:id", get(handler::get_account))
        .route("/api/accounts", get(handler::get_account))
        .route("/api/accounts", post(handler::create_account))
        .route("/api/users/login", post(handler::validate_user))
        .route("/api/users/:id", get(handler::get_user))
        .route("/api/users", post(handler::create_user))
        .with_state(app_state);
    router
}
