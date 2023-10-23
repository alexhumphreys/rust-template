use crate::handlers;
use axum::{routing::get, Router};
use tower_http::services::ServeDir;

pub fn router() -> Router {
    let public = Router::new()
        .nest_service("/assets", ServeDir::new("assets"))
        .route("/styles", get(handlers::styles))
        .route("/greet/:name", get(handlers::greet))
        .route("/login", get(handlers::login).post(handlers::handle_login))
        .route("/perm", get(handlers::perm));
    public
}
