use crate::error::Error;
use axum::{
    extract::rejection::JsonRejection, extract::FromRequest, http::StatusCode,
    response::IntoResponse,
};
use serde::Deserialize;
use serde::Serialize;
use uuid::Uuid;

#[derive(Deserialize, Debug, Default)]
pub struct FilterOptions {
    pub page: Option<usize>,
    pub limit: Option<usize>,
}

#[derive(Deserialize, Debug)]
pub struct ParamOptions {
    pub id: Uuid,
}

#[derive(Deserialize, Debug)]
pub struct PathId {
    pub id: Uuid,
}

#[derive(Deserialize, Debug)]
pub struct CreateAccount {
    pub name: String,
    pub credential: String,
}
