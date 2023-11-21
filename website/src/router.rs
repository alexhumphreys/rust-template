use crate::{auth, protected_routes, proxy_routes, public_routes};
use axum::Router;

pub async fn routes() -> Router {
    let (auth_session_layer, session_layer) = auth::make_auth_session_layer().await;

    Router::new()
        .merge(protected_routes::router())
        .merge(public_routes::router())
        // include authentication session middleware
        .layer(auth_session_layer)
        // include session storage
        .layer(session_layer)
        // include proxy after the session auth
        .merge(proxy_routes::router())
}
