use crate::{app_state, handlers};
use axum::{routing::get, Router};
use tower_http::services::ServeDir;

pub async fn router() -> Router {
    let public = Router::new()
        .nest_service("/assets", ServeDir::new("assets"))
        .route("/styles", get(handlers::styles))
        .route("/greet/:name", get(handlers::greet))
        .route("/login", get(handlers::login).post(handlers::handle_login))
        .route("/about", get(handlers::about_page))
        .route("/perm", get(handlers::perm))
        .with_state(app_state::create_app_state().await);
    public
}
