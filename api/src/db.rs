use crate::AppState;
use anyhow::{Context, Result};
use argon2::password_hash::SaltString;
use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use axum::extract::{Query, State};
use axum::http::StatusCode;
use rand;
use secrecy::ExposeSecret;
use shared::schema::LoginPayload;
use shared::{
    error::Error,
    model::{AccountModel, ClientModel, UserModel, UserShortModel, UserTransportModel},
    schema,
    tracing::make_otel_db_span,
};
use sqlx::Execute;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::sync::Arc;
use tracing::{self, Instrument};
use uuid::Uuid;

pub async fn get_client_list(
    opts: Option<Query<schema::FilterOptions>>,
    State(data): State<Arc<AppState>>,
) -> Result<Vec<ClientModel>, Error> {
    let Query(opts) = opts.unwrap_or_default();

    let limit = opts.limit.unwrap_or(10);
    let offset = (opts.page.unwrap_or(1) - 1) * limit;

    let query = sqlx::query_as!(
        ClientModel,
        "SELECT * FROM clients ORDER by id LIMIT $1 OFFSET $2",
        limit as i32,
        offset as i32
    );

    let sql = query.sql().clone();
    let query_result = query
        .fetch_all(&data.db)
        .instrument(make_otel_db_span("SELECT", sql))
        .await?;

    Ok(query_result)
}

pub async fn get_account(
    account_id: Uuid,
    State(data): State<Arc<AppState>>,
) -> Result<AccountModel, Error> {
    tracing::debug!("Searching for account id {}", account_id.to_string());
    let query = sqlx::query_as!(
        AccountModel,
        "SELECT * FROM accounts WHERE id = $1",
        account_id
    );
    let sql = query.sql().clone();
    let query_result = query
        .fetch_one(&data.db)
        .instrument(make_otel_db_span("SELECT", sql))
        .await?;
    //.with_context(|| format!("No value found for: {}", account_id.to_string()))?;

    Ok(query_result)
}

pub async fn get_account_by_name(
    account_name: String,
    State(data): State<Arc<AppState>>,
) -> Result<AccountModel, Error> {
    tracing::debug!("Searching for account by name");
    let query = sqlx::query_as!(
        AccountModel,
        "SELECT * FROM accounts WHERE name = $1",
        account_name
    );
    let sql = query.sql().clone();
    let query_result = query
        .fetch_one(&data.db)
        .instrument(make_otel_db_span("SELECT", sql))
        .await?;
    //.with_context(|| format!("No value found for: {}", account_id.to_string()))?;

    Ok(query_result)
}

pub async fn create_account(
    account: schema::CreateAccount,
    State(data): State<Arc<AppState>>,
) -> Result<AccountModel, Error> {
    tracing::debug!("creating account");
    let query = sqlx::query_as!(
        AccountModel,
        "INSERT INTO accounts(name, credential) VALUES ($1, $2) RETURNING *",
        account.name,
        account.credential
    );
    let sql = query.sql().clone();
    let query_result = query
        .fetch_one(&data.db)
        .instrument(make_otel_db_span("INSERT", sql))
        .await?;

    Ok(query_result)
}

pub async fn put_account(
    id: Uuid,
    account: schema::CreateAccount,
    State(data): State<Arc<AppState>>,
) -> Result<AccountModel, Error> {
    tracing::debug!("creating account");
    let query = sqlx::query_as!(
        AccountModel,
        "UPDATE accounts SET name = $1, credential = $2 WHERE id = $3 RETURNING *",
        account.name,
        account.credential,
        id
    );
    let sql = query.sql().clone();
    let query_result = query
        .fetch_one(&data.db)
        .instrument(make_otel_db_span("INSERT", sql))
        .await?;

    Ok(query_result)
}

#[derive(thiserror::Error, Debug)]
pub enum AuthError {
    #[error("Invalid credentials.")]
    InvalidCredentials(#[source] anyhow::Error),
    #[error(transparent)]
    UnexpectedError(#[from] anyhow::Error),
}

fn calculate_hash<T: Hash>(t: &T) -> u64 {
    let mut s = DefaultHasher::new();
    t.hash(&mut s);
    s.finish()
}

pub async fn validate_credentials(
    credentials: LoginPayload,
    State(data): State<Arc<AppState>>,
) -> Result<UserTransportModel, Error> {
    let query = sqlx::query_as!(
        UserModel,
        r#"
        SELECT id, name, password_hash
        FROM users
        WHERE name = $1
        "#,
        credentials.name,
    );
    let sql = query.sql().clone();
    let user = query
        .fetch_one(&data.db)
        .instrument(make_otel_db_span("SELECT", sql))
        .await
        .map_err(|_| Error::Unauthorized)?;

    let expected_password_hash = PasswordHash::new(&user.password_hash)
        .context("Failed to parse hash in PHC string format.")?;

    // TODO move to thread
    Argon2::default()
        .verify_password(
            credentials.password.expose_secret().as_bytes(),
            &expected_password_hash,
        )
        .map_err(|_| Error::Unauthorized)?;

    Ok(UserTransportModel {
        id: user.id,
        name: user.name,
    })
}

pub async fn create_user(
    credentials: LoginPayload,
    State(data): State<Arc<AppState>>,
) -> Result<UserShortModel, Error> {
    let password_hash = generate_hash(&credentials).await;

    let user = sqlx::query_as!(
        UserShortModel,
        "INSERT INTO users(name, password_hash) VALUES ($1, $2) RETURNING id, name",
        credentials.name,
        password_hash
    )
    .fetch_one(&data.db)
    .await
    .context("Failed to create user.")?;

    Ok(user)
}

async fn generate_hash(credentials: &LoginPayload) -> String {
    let salt = SaltString::generate(&mut rand::thread_rng());
    let password_hash = Argon2::default()
        .hash_password(credentials.password.expose_secret().as_bytes(), &salt)
        .unwrap()
        .to_string();

    password_hash
}

pub async fn get_user(
    user_id: Uuid,
    State(data): State<Arc<AppState>>,
) -> Result<UserTransportModel, Error> {
    let query = sqlx::query_as!(
        UserTransportModel,
        "SELECT id, name FROM users WHERE id = $1",
        user_id
    );
    let sql = query.sql().clone();
    let user = query
        .fetch_one(&data.db)
        .instrument(make_otel_db_span("SELECT", sql))
        .await
        .context("Failed to get user.")?;
    Ok(user)
}
