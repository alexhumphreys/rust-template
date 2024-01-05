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
use urlencoding::encode;

/// Helper to create a random string 30 chars long.
pub fn random_state_string() -> String {
    use rand::distributions::{Alphanumeric, DistString};
    let string = Alphanumeric.sample_string(&mut rand::thread_rng(), 30);
    println!("Random string: {}", string);
    string
}

/// Configuration state for Auth0, including the client secret, which
/// must be kept private.
struct AuthSettings {
    client_id: String,
    client_secret: String,
    redirect_uri: String,
    auth0_domain: String,
}
impl AuthSettings {
    /// Given a state param, build a url String that our /auth0 redirect handler can use.
    pub fn authorize_endpoint_url(&self, state: &str) -> String {
        format!(
            "https://{}/authorize?response_type=code&client_id={}&redirect_uri={}&scope=openid%20profile&state={}",
            self.auth0_domain,
            self.client_id,
            encode(&self.redirect_uri),
            state,
        )
    }
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
    // cookies.add(Cookie::new("state", state.clone()));

    todo!()
}
