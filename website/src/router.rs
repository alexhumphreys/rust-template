use crate::{auth, auth0, protected_routes, proxy_routes, public_routes};
use axum::Router;

pub async fn routes() -> Router {
    let (auth_session_layer, session_layer) = auth::make_auth_session_layer().await;

    Router::new()
        .merge(protected_routes::router().await)
        .merge(public_routes::router().await)
        // include authentication session middleware
        .layer(auth_session_layer)
        // include session storage
        .merge(auth0::router().await)
        .layer(session_layer)
        // include proxy after the session auth
        .merge(proxy_routes::router())
}
