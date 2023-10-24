use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, FromRow, Deserialize, Serialize)]
#[allow(non_snake_case)]
pub struct ClientModel {
    pub id: Uuid,
    pub name: String,
}

#[derive(Debug, FromRow, Deserialize, Serialize)]
pub struct AccountModel {
    pub id: Uuid,
    pub name: String,
    pub credential: String,
}

#[derive(Debug, FromRow, Deserialize, Serialize)]
pub struct UserModel {
    pub id: Uuid,
    pub name: String,
    pub password_hash: String,
}

#[derive(Debug, FromRow, Deserialize, Serialize)]
pub struct UserShortModel {
    pub id: Uuid,
    pub name: String,
}

#[derive(Debug, FromRow, Deserialize, Serialize, PartialEq, Eq)]
pub struct UserTransportModel {
    pub id: Uuid,
    pub name: String,
}
