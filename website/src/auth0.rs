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
use axum::{
    debug_handler,
    extract::{Query, State},
    response::Redirect,
    routing::get,
    Router,
};
use serde::{Deserialize, Serialize};
use serde_json::ser::to_vec;
use shared::error;
use std::sync::Arc;
use urlencoding::encode;

/// Helper to create a random string 30 chars long.
pub fn random_state_string() -> String {
    use rand::distributions::{Alphanumeric, DistString};
    let string = Alphanumeric.sample_string(&mut rand::thread_rng(), 30);
    println!("Random string: {}", string);
    string
}

pub fn get_settings() -> AuthSettings {
    // read settings from environment variables
    let client_id = std::env::var("AUTH0_CLIENT_ID").expect("AUTH0_CLIENT_ID must be set");
    let client_secret =
        std::env::var("AUTH0_CLIENT_SECRET").expect("AUTH0_CLIENT_SECRET must be set");
    let redirect_uri = std::env::var("AUTH0_REDIRECT_URI").expect("AUTH0_REDIRECT_URI must be set");
    let auth0_domain =
        std::env::var("AUTH0_DOMAIN").expect("AUTH0_DOMAIN must be set, e.g. 'example.auth0.com'");
    AuthSettings {
        client_id,
        client_secret,
        redirect_uri,
        auth0_domain,
    }
}

/// Configuration state for Auth0, including the client secret, which
/// must be kept private.
#[derive(Debug)]
pub struct AuthSettings {
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
    pub fn token_endpoint_url(&self) -> String {
        format!("https://{}/oauth/token", self.auth0_domain)
    }
    /// Builds a TokenRequest from an authorization code and
    /// Auth0 config values.
    pub fn token_request(&self, code: &str) -> TokenRequest {
        TokenRequest {
            grant_type: String::from("authorization_code"),
            client_id: self.client_id.clone(),
            client_secret: self.client_secret.clone(),
            code: code.to_string(),
            redirect_uri: self.redirect_uri.clone(),
        }
    }
}

pub async fn router() -> Router {
    let auth0 = Router::new()
        .route("/auth0_login", get(handlers::styles))
        .route("/loggedin", get(handlers::styles))
        .route("/auth0", get(auth0_redirect))
        .route("/callback", get(callback))
        .with_state(app_state::create_app_state().await);
    auth0
}

#[debug_handler]
pub async fn auth0_redirect(
    State(data): State<Arc<app_state::AppState>>,
) -> Result<Redirect, error::Error> {
    let state = random_state_string();
    // cookies.add(Cookie::new("state", state.clone()));
    let uri = data.auth0.authorize_endpoint_url(&state);
    println!("{:?}", uri);
    Ok(Redirect::to(&uri))
}

#[derive(Deserialize)]
pub struct CallbackSchema {
    code: String,
    state: String,
}

/// Send TokenRequest to the Auth0 /oauth/token endpoint.
#[derive(Serialize, Deserialize)]
struct TokenRequest {
    grant_type: String,
    client_id: String,
    client_secret: String,
    code: String,
    redirect_uri: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct TokenResponse {
    access_token: String,
    expires_in: u32,
    id_token: String,
    token_type: String,
}

#[debug_handler]
pub async fn callback(
    auth0: Query<CallbackSchema>,
    State(data): State<Arc<app_state::AppState>>,
) -> Result<Redirect, error::Error> {
    // TODO check state from cookie
    let tr = data.auth0.token_request(&auth0.code);
    let client = reqwest::Client::new();
    let resp: TokenResponse = client
        .post(data.auth0.token_endpoint_url())
        .header("Content-Type", "application/json")
        .body(to_vec(&tr).unwrap())
        .send()
        .await?
        .json()
        .await?;

    println!("{:?}", resp);
    Ok(Redirect::to("/loggedin"))
}
