use crate::auth::{AuthSessionType, NullPool, User};

use askama::Template;
use axum::{
    body::{Body, Bytes},
    debug_handler, extract,
    extract::{Extension, Json, Path, Query, State, TypedHeader},
    http::Method,
    http::{uri::Uri, Request},
    response::{Html, IntoResponse, Redirect},
};
use axum_session_auth::{Auth, Rights};
use reqwest_middleware::ClientWithMiddleware;
use serde::Deserialize;
use shared::{client, schema::LoginPayload2};
use std::collections::HashMap;
use tera::Tera;
use uuid::Uuid;

#[derive(Template)]
#[template(path = "login.html")]
struct LoginTemplate {}

pub async fn login() -> impl IntoResponse {
    let template = LoginTemplate {};
    template
}

fn tera_include() -> Tera {
    let tera = Tera::new("tera/**/*").unwrap();
    tera
}
fn common_context() -> tera::Context {
    let mut context = tera::Context::new();
    context.insert("title", "axum-tera");
    context
}
pub async fn about_page() -> Html<String> {
    let tera = tera_include();
    let mut context = common_context();
    context.insert("page_title", "About");
    context.insert("message", "This is About page.");
    let output = tera.render("about.html", &context);
    Html(output.unwrap())
}

#[derive(Template)]
#[template(path = "hello.html")]
struct HelloTemplate {
    name: String,
}
pub async fn greet_protected(auth: AuthSessionType) -> impl IntoResponse {
    let current_user = auth.current_user.clone().unwrap_or_default();
    let template = HelloTemplate {
        name: current_user.username,
    };
    template
}

pub async fn greet(extract::Path(name): extract::Path<String>) -> impl IntoResponse {
    let template = HelloTemplate { name };
    template
}

#[derive(Template)]
#[template(path = "styles.html")]
struct StylesTemplate {}

pub async fn styles() -> impl IntoResponse {
    let template = StylesTemplate {};
    template
}

#[derive(Deserialize, Debug)]
#[allow(dead_code)]
pub struct Input {
    name: String,
    password: String,
}

#[debug_handler]
pub async fn handle_login(
    auth: AuthSessionType,
    extract::Form(input): extract::Form<Input>,
) -> Redirect {
    let login_payload = LoginPayload2 {
        name: input.name,
        password: input.password,
    };
    match client::auth_user(login_payload, None).await {
        Ok(user) => {
            auth.login_user(Some(user.id));
            Redirect::to("/perm")
        }
        Err(e) => {
            tracing::error!("TODO REMOVE error {:?}", e);
            Redirect::to("/login")
        }
    }
}

pub async fn perm(method: Method, auth: AuthSessionType) -> String {
    let current_user = auth.current_user.clone().unwrap_or_default();

    //lets check permissions only and not worry about if they are anon or not
    if !Auth::<User, Option<Uuid>, NullPool>::build([Method::GET], false)
        .requires(Rights::any([
            Rights::permission("Category::View"),
            Rights::permission("Admin::View"),
        ]))
        .validate(&current_user, &method, None)
        .await
    {
        return format!(
            "User {}, Does not have permissions needed to view this page please login",
            current_user.username
        );
    }

    format!(
        "User id {:?} and name {:?} has Permissions needed. Here are the Users permissions: {:?}",
        current_user.id, current_user.username, current_user.permissions
    )
}

// Proxy Routes
async fn proxy_handler(
    State(client): State<ClientWithMiddleware>,
    method: Method,
    query: Query<HashMap<String, String>>,
    body: String,
    path: Path<String>,
) -> Result<String, axum::http::StatusCode> {
    todo!()
    /*
    let mut request_builder = client.request(method, &url);
    if let Some(token) = current_user.token {
        request_builder = request_builder.header("Authorization", format!("Bearer {}", token));
    }
    let response = request_builder
        .body(req.into_body())
        .send()
        .await
        .map_err(|e| {
            tracing::error!("TODO REMOVE error {:?}", e);
            axum::http::StatusCode::INTERNAL_SERVER_ERROR
        })?;
    Ok(response)
    */
}
