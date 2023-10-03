use axum::{Json, Router};
use oasgen::{openapi, OaSchema, Server};
use serde::{Deserialize, Serialize};

#[derive(OaSchema, Deserialize)]
pub struct SendCode {
    pub mobile: String,
}

#[derive(Serialize, OaSchema, Debug)]
pub struct SendCodeResponse {
    pub found_account: bool,
}

#[openapi]
async fn send_code(_body: Json<SendCode>) -> Json<SendCodeResponse> {
    Json(SendCodeResponse {
        found_account: false,
    })
}

pub fn oa_generated_server() -> Router<()> {
    Server::axum()
        .post("/send-code", send_code)
        .write_and_exit_if_env_var_set("openapi.yaml") // set OASGEN_WRITE_SPEC=1
        .freeze()
        .into_router()
}
