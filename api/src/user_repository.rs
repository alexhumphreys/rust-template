use crate::db_init::Db;
use anyhow::Context;
use argon2::{password_hash::SaltString, Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use axum::async_trait;
use mockall::automock;
use secrecy::{ExposeSecret, Secret};
use shared::{
    error::Error,
    model::{UserModel, UserTransportModel},
    schema::LoginPayload,
    tracing::make_otel_db_span,
};
use sqlx::Execute;
use tokio::task;
use tracing::{self, Instrument};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct UserRepoImpl {
    pool: Db,
}
impl UserRepoImpl {
    pub fn new(pool: Db) -> Self {
        Self { pool }
    }
}

#[automock]
#[async_trait]
pub trait UserRepo {
    async fn get_user(&self, user_id: Uuid) -> Result<UserTransportModel, Error>;
    async fn create_user(&self, credentials: LoginPayload) -> Result<UserTransportModel, Error>;
    async fn validate_credentials(
        &self,
        credentials: LoginPayload,
    ) -> Result<UserTransportModel, Error>;
}

#[async_trait]
impl UserRepo for UserRepoImpl {
    async fn validate_credentials(
        &self,
        credentials: LoginPayload,
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
            .fetch_one(&*self.pool)
            .instrument(make_otel_db_span("SELECT", sql))
            .await
            .map_err(|_| Error::Unauthorized)?;

        task::spawn_blocking(move || {
            verify_password_hash(user.password_hash, credentials.password)
        })
        .await
        .context("Failed to spawn blocking task.")
        .map_err(|_| Error::InternalServerError)??;

        let return_user = UserTransportModel {
            id: user.id,
            name: user.name,
        };

        Ok(return_user)
    }
    async fn create_user(&self, credentials: LoginPayload) -> Result<UserTransportModel, Error> {
        let password_hash = generate_hash(&credentials).await;

        let user = sqlx::query_as!(
            UserTransportModel,
            "INSERT INTO users(name, password_hash) VALUES ($1, $2) RETURNING id, name",
            credentials.name,
            password_hash
        )
        .fetch_one(&*self.pool)
        .await
        .context("Failed to create user.")?;

        Ok(user)
    }

    async fn get_user(&self, user_id: Uuid) -> Result<UserTransportModel, Error> {
        let query = sqlx::query_as!(
            UserTransportModel,
            "SELECT id, name FROM users WHERE id = $1",
            user_id
        );
        let sql = query.sql().clone();
        let user = query
            .fetch_one(&*self.pool)
            .instrument(make_otel_db_span("SELECT", sql))
            .await
            .context("Failed to get user.")?;
        Ok(user)
    }
}

async fn generate_hash(credentials: &LoginPayload) -> String {
    let salt = SaltString::generate(&mut rand::thread_rng());
    let password_hash = Argon2::default()
        .hash_password(credentials.password.expose_secret().as_bytes(), &salt)
        .unwrap()
        .to_string();

    password_hash
}

#[tracing::instrument(
    name = "Verify password hash",
    skip(expected_password_hash, password_candidate)
)]
fn verify_password_hash(
    expected_password_hash: String,
    password_candidate: Secret<String>,
) -> Result<(), Error> {
    let expected_password_hash = PasswordHash::new(&expected_password_hash)
        .context("Failed to parse hash in PHC string format.")
        .map_err(|_| Error::InternalServerError)?;

    Argon2::default()
        .verify_password(
            password_candidate.expose_secret().as_bytes(),
            &expected_password_hash,
        )
        .context("Invalid password.")
        .map_err(|_| Error::Unauthorized)
}
