// fn get_routes() -> Vec<rocket::Route> {
//    routes![
//        login,
//        logged_in,
//        auth0_redirect,
//        auth0_callback,
//        home,
//        home_redirect,
//        static_files
//    ]
//}

use crate::{app_state, handlers};
use axum::{debug_handler, response::Redirect, routing::get, Router};
use shared::error;
use tower_http::services::ServeDir;

/// Helper to create a random string 30 chars long.
pub fn random_state_string() -> String {
    use rand::distributions::{Alphanumeric, DistString};
    let string = Alphanumeric.sample_string(&mut rand::thread_rng(), 30);
    println!("Random string: {}", string);
    string
}

pub async fn router() -> Router {
    let auth0 = Router::new()
        .route("/login", get(handlers::styles))
        .route("/loggedin", get(handlers::greet))
        .route("/auth0", get(auth0_redirect))
        .route("/callback", get(handlers::styles));
    auth0
}

#[debug_handler]
pub async fn auth0_redirect() -> Result<Redirect, error::Error> {
    let state = random_state_string();

    todo!()
}
