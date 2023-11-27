use schemars::JsonSchema;
use secrecy::Secret;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Deserialize, Debug, Default, JsonSchema)]
pub struct FilterOptions {
    pub page: Option<usize>,
    pub limit: Option<usize>,
}

#[derive(Deserialize, Debug, JsonSchema)]
pub struct ParamOptions {
    pub id: Uuid,
}

#[derive(Deserialize, Debug, JsonSchema)]
pub struct PathId {
    pub id: Uuid,
}

#[derive(Deserialize, Debug, JsonSchema)]
pub struct PathName {
    pub name: String,
}

#[derive(Deserialize, Debug, JsonSchema)]
pub struct CreateAccount {
    pub name: String,
    pub credential: String,
}

#[derive(Deserialize, Debug)]
pub struct LoginPayload {
    pub name: String,
    pub password: Secret<String>,
}

// TODO hack hack
#[derive(Deserialize, Debug, Serialize, JsonSchema)]
pub struct LoginPayload2 {
    pub name: String,
    pub password: String,
}

#[derive(Deserialize, Debug, JsonSchema)]
pub struct CreateClient {
    pub name: String,
    pub user_id: Uuid,
}

#[derive(Serialize, Deserialize, Debug, JsonSchema)]
pub struct ValidateToken {
    pub token: String,
}
