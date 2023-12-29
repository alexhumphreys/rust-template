use crate::{
    app_state::AppState,
    auth::{AuthSessionType, NullPool, User},
};

use askama::Template;
use axum::{
    debug_handler, extract,
    extract::{Path, Query, State},
    http::Method,
    response::{Html, IntoResponse, Redirect},
};
use axum_session_auth::{Auth, Rights};
use fluent_templates::{ArcLoader, FluentLoader};
use reqwest_middleware::ClientWithMiddleware;
use serde::Deserialize;
use serde_json::json;
use shared::{client, schema::LoginPayload2};
use std::{collections::HashMap, sync::Arc};
use uuid::Uuid;

#[derive(Template)]
#[template(path = "login.html")]
struct LoginTemplate {}

pub async fn login() -> impl IntoResponse {
    let template = LoginTemplate {};
    template
}

fluent_templates::static_loader! {
    // Declare our `StaticLoader` named `LOCALES`.
    static LOCALES = {
        // The directory of localisations and fluent resources.
        locales: "locales",
        // The language to falback on if something is not present.
        fallback_language: "en-US",
    };
}

pub async fn about_page(State(data): State<Arc<AppState>>) -> Html<String> {
    let data0 = json!({
        "title": "example 0",
        "parent": "base0",
        "lang": "de-DE",
    });
    Html(data.handlebars.render("template2", &data0).unwrap())
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
