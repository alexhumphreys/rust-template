use crate::auth::{AuthSessionType, NullPool, User};

use askama::Template;
use axum::{
    debug_handler, extract,
    http::Method,
    response::{IntoResponse, Redirect},
};
use axum_session_auth::{Auth, Rights};
use serde::Deserialize;
use shared::{client, schema::LoginPayload2};
use uuid::Uuid;

#[derive(Template)]
#[template(path = "login.html")]
struct LoginTemplate {}

pub async fn login() -> impl IntoResponse {
    let template = LoginTemplate {};
    template
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
