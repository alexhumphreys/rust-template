use crate::{auth, handlers};
use axum::{middleware, routing::get, Router};

pub fn router() -> Router {
    let protected = Router::new()
        .route("/greet-protected", get(handlers::greet_protected))
        .route_layer(middleware::from_fn(auth::session_auth));
    protected
}
