use serde::Deserialize;
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
