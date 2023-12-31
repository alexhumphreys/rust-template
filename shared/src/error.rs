use anyhow;
use axum;
use axum::extract::rejection::JsonRejection;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Json, Response};
use reqwest;
use reqwest_middleware;
use serde::{Deserialize, Serialize};

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Request lacks valid authentication credentials for the requested resource")]
    Unauthorized,

    #[error("Server understands the request but refuses to authorize it")]
    Forbidden,

    #[error("Not found")]
    NotFound,

    #[error("Bad request")]
    BadRequest,

    #[error("Internal server error")]
    InternalServerError,

    #[error("An internal server error occurred")]
    Anyhow(#[from] anyhow::Error),

    #[error("A database error occurred")]
    Sqlx(#[from] sqlx::Error),

    #[error("A reqwest error occurred")]
    Reqwest(#[from] reqwest::Error),

    #[error("A reqwest_middleware error occurred")]
    ReqwestMiddelware(#[from] reqwest_middleware::Error),

    #[error("An axum error occurred")]
    Axum(#[from] axum::Error),

    #[error("An axum error occurred")]
    SerdeJson(#[from] serde_json::Error),
}

impl Error {
    fn code_detail(&self) -> (StatusCode, String) {
        let code = match self {
            Error::Unauthorized => StatusCode::UNAUTHORIZED,
            Error::Forbidden => StatusCode::FORBIDDEN,
            Error::NotFound => StatusCode::NOT_FOUND,
            Error::BadRequest => StatusCode::BAD_REQUEST,
            Error::InternalServerError => StatusCode::INTERNAL_SERVER_ERROR,
            Error::Anyhow(e) => {
                tracing::error!("Anyhow error: {:?}", e);
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Internal server error".to_string(),
                );
            }
            Error::Reqwest(e) => {
                tracing::error!("Reqwest error: {:?}", e);
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Internal server error".to_string(),
                );
            }
            Error::SerdeJson(e) => {
                tracing::error!("serde_json error: {:?}", e);
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Internal server error".to_string(),
                );
            }
            Error::Axum(e) => {
                tracing::error!("Axum error: {:?}", e);
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Internal server error".to_string(),
                );
            }
            Error::Sqlx(e) => match e {
                sqlx::Error::RowNotFound => {
                    return (
                        StatusCode::NOT_FOUND,
                        "Request returned no results".to_string(),
                    )
                }
                sqlx::Error::Database(err) => match err.kind() {
                    sqlx::error::ErrorKind::UniqueViolation => {
                        return (StatusCode::CONFLICT, format!("Conflict: {}", err.message()));
                    }
                    _ => {
                        tracing::error!("Sqlx database error: {:?}", err);
                        return (
                            StatusCode::INTERNAL_SERVER_ERROR,
                            "Internal server error".to_string(),
                        );
                    }
                },
                _ => {
                    tracing::error!("Sqlx error: {:?}", e);
                    return (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        "Internal server error".to_string(),
                    );
                }
            },
            _ => {
                tracing::error!("Unknown internal server error");
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Internal server error".to_string(),
                );
            }
        };
        (code, self.to_string())
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct ErrorJson {
    status: String,
    source: Option<String>,
    title: String,
    detail: String,
}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        let (code, detail) = self.code_detail();

        let res: Vec<ErrorJson> = [ErrorJson {
            status: code.as_str().to_string(),
            source: None,
            title: code.to_string(),
            detail,
        }]
        .to_vec();

        (code, Json(res)).into_response()
    }
}

// We implement `From<JsonRejection> for ApiError`
impl From<JsonRejection> for Error {
    fn from(rejection: JsonRejection) -> Self {
        Error::BadRequest
        //Self {
        //status: rejection.status(),
        //message: rejection.body_text(),
        //}
    }
}
