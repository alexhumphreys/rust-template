use crate::{app_state::create_app_state, auth, handlers};
use axum::{middleware, routing::get, Router};

pub async fn router() -> Router {
    let protected = Router::new()
        .route("/greet-protected", get(handlers::greet_protected))
        .route_layer(middleware::from_fn(auth::session_auth))
        .with_state(create_app_state().await);
    protected
}
