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
use frank_jwt::{decode, Algorithm};
use serde::{Deserialize, Serialize};
use serde_json::{ser::to_vec, Value};
use shared::error;
use std::sync::Arc;
use urlencoding::encode;

#[derive(Deserialize)]
pub struct CallbackSchema {
    code: String,
    state: String,
}

/// Send TokenRequest to the Auth0 /oauth/token endpoint.
#[derive(Serialize, Deserialize)]
pub struct TokenRequest {
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

/// Holds deserialized data from the /oauth/token endpoint. Use the fields
/// of this struct for validation.
#[derive(Debug, Serialize, Deserialize)]
struct Auth0JWTPayload {
    email: String,
    sub: String,
    exp: i64,
    iss: String,
    aud: String,
}

#[derive(Debug)]
struct Auth0CertInfo {
    pem_pk: Vec<u8>,
    der_pk: Vec<u8>,
    der_cert: Vec<u8>,
}

/// Helper to create a random string 30 chars long.
pub fn random_state_string() -> String {
    use rand::distributions::{Alphanumeric, DistString};
    let string = Alphanumeric.sample_string(&mut rand::thread_rng(), 30);
    println!("Random string: {}", string);
    string
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
            "https://{}/authorize?response_type=code&client_id={}&redirect_uri={}&scope=openid%20email%20profile&state={}",
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
    pub fn from_env() -> AuthSettings {
        // read settings from environment variables
        let client_id = std::env::var("AUTH0_CLIENT_ID").expect("AUTH0_CLIENT_ID must be set");
        let client_secret =
            std::env::var("AUTH0_CLIENT_SECRET").expect("AUTH0_CLIENT_SECRET must be set");
        let redirect_uri =
            std::env::var("AUTH0_REDIRECT_URI").expect("AUTH0_REDIRECT_URI must be set");
        let auth0_domain = std::env::var("AUTH0_DOMAIN")
            .expect("AUTH0_DOMAIN must be set, e.g. 'example.auth0.com'");
        AuthSettings {
            client_id,
            client_secret,
            redirect_uri,
            auth0_domain,
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
    let certs = populate_certs(&data.auth0.auth0_domain).await?;
    println!("{:?}", certs);
    let payload = decode_and_validate_jwt(
        certs.pem_pk,
        &resp.id_token,
        &data.auth0.client_id,
        &data.auth0.auth0_domain,
    );
    println!("{:?}", payload);
    Ok(Redirect::to("/loggedin"))
}

fn decode_and_validate_jwt(
    pub_key: Vec<u8>,
    jwt: &str,
    aud: &str,
    auth0_domain: &str,
) -> Result<Auth0JWTPayload, error::Error> {
    // TODO better error handling
    let (header, payload) = decode(
        &jwt.to_string(),
        &String::from_utf8(pub_key).expect("pk is not valid UTF-8"),
        Algorithm::RS256,
        &frank_jwt::ValidationOptions::default(),
    )
    .unwrap();
    let payload: Auth0JWTPayload = serde_json::from_value(payload).unwrap();
    if payload.aud != aud.to_string() {
        todo!()
    };
    if payload.iss != format!("https://{}/", auth0_domain) {
        todo!()
    };
    Ok(payload)
}

async fn populate_certs(auth0_domain: &str) -> Result<Auth0CertInfo, error::Error> {
    let client = reqwest::Client::new();
    let cert_endpoint = format!("https://{}/pem", auth0_domain);
    let pem_cert: String = client.get(cert_endpoint).send().await?.text().await?;
    // transform cert into X509 struct
    use openssl::x509::X509;
    let cert = X509::from_pem(pem_cert.as_bytes()).expect("x509 parse failed");
    let pk = cert.public_key().unwrap();
    // extract public keys and cert in pem and der
    let pem_pk = pk.public_key_to_pem().unwrap();
    let der_pk = pk.public_key_to_der().unwrap();
    let der_cert = cert.to_der().unwrap();
    Ok(Auth0CertInfo {
        pem_pk,
        der_pk,
        der_cert,
    })
}

/*
 * example of how to parse the error responses from auth0
 *
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
struct CommonFields {
    common_field1: String,
    common_field2: i32,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(tag = "type")]
enum MyEnum {
    #[serde(rename = "schema1")]
    Schema1 {
        #[serde(flatten)]
        common: CommonFields,
        schema1_field: String,
    },
    #[serde(rename = "schema2")]
    Schema2 {
        #[serde(flatten)]
        common: CommonFields,
        schema2_field: i64,
    },
}

fn main() {
    let json1 = r#"
        {
            "type": "schema1",
            "common_field1": "value1",
            "common_field2": 42,
            "schema1_field": "schema1_value"
        }
    "#;

    let json2 = r#"
        {
            "type": "schema2",
            "common_field1": "value2",
            "common_field2": 84,
            "schema2_field": 123
        }
    "#;

    let result1: Result<MyEnum, _> = serde_json::from_str(json1);
    let result2: Result<MyEnum, _> = serde_json::from_str(json2);

    println!("{:?}", result1);
    println!("{:?}", result2);
}
*/
