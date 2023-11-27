use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, FromRow, Deserialize, Serialize, JsonSchema)]
#[allow(non_snake_case)]
pub struct ClientModel {
    pub id: Uuid,
    pub name: String,
    pub user_id: Uuid,
    pub token: String,
}

#[derive(Debug, FromRow, Deserialize, Serialize, JsonSchema)]
pub struct AccountModel {
    pub id: Uuid,
    pub name: String,
    pub credential: String,
}

#[derive(Debug, FromRow, Deserialize, Serialize, JsonSchema)]
pub struct UserModel {
    pub id: Uuid,
    pub name: String,
    pub password_hash: String,
}

#[derive(Debug, FromRow, Deserialize, Serialize, JsonSchema)]
pub struct UserShortModel {
    pub id: Uuid,
    pub name: String,
}

#[derive(Debug, FromRow, Deserialize, Serialize, JsonSchema, PartialEq, Eq)]
pub struct UserTransportModel {
    pub id: Uuid,
    pub name: String,
}

#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct DataWrapper<T>
where
    T: Serialize + JsonSchema,
{
    pub data: T,
}
