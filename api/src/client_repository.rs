use crate::db_init::Db;
use anyhow::Context;
use axum::async_trait;
use mockall::automock;
use rand::distributions::{Alphanumeric, DistString};
use rand::rngs::OsRng;
use rand::{prelude::SliceRandom, CryptoRng, Rng, RngCore};
use shared::{error::Error, model::ClientModel, tracing::make_otel_db_span};
use sqlx::Execute;
use tracing::{self, Instrument};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct ClientRepoImpl {
    pool: Db,
}
impl ClientRepoImpl {
    pub fn new(pool: Db) -> Self {
        Self { pool }
    }
}

#[automock]
#[async_trait]
pub trait ClientRepo {
    async fn create_client(&self, user_id: Uuid) -> Result<ClientModel, Error>;
    async fn get_client(&self, token: String) -> Result<ClientModel, Error>;
}

#[async_trait]
impl ClientRepo for ClientRepoImpl {
    async fn create_client(&self, user_id: Uuid) -> Result<ClientModel, Error> {
        let mut rng = OsRng;
        let random_string = generate_random_string(&mut rng, 18);
        println!("{}", random_string);
        todo!()
    }

    async fn get_client(&self, token: String) -> Result<ClientModel, Error> {
        let query = sqlx::query_as!(ClientModel, "SELECT * FROM clients WHERE token = $1", token);
        let sql = query.sql().clone();
        let client = query
            .fetch_one(&*self.pool)
            .instrument(make_otel_db_span("SELECT", sql))
            .await
            .context("Failed to get client.")?;
        Ok(client)
    }
}

fn generate_random_string<R: Rng + CryptoRng>(rng: &mut R, length: usize) -> String {
    const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ\
                            abcdefghijklmnopqrstuvwxyz\
                            0123456789";

    let random_bytes: Vec<u8> = (0..length).map(|_| *CHARSET.choose(rng).unwrap()).collect();

    String::from_utf8(random_bytes).unwrap()
}
