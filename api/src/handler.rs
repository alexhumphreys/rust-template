use std::sync::Arc;

use crate::{model::ClientModel, schema::FilterOptions, AppState};
use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use sqlx::Execute;
use tracing::{self, Instrument};

fn make_otel_span(db_operation: &str, db_statement: &str) -> tracing::Span {
    // NO parsing of statement to extract information, not recommended by Specification and time-consuming
    // warning: providing the statement could leek information
    tracing::trace_span!(
        target: tracing_opentelemetry_instrumentation_sdk::TRACING_TARGET,
        "DB request",
        service.name = "api-postgres",
        db.system = "postgresql",
        db.statement = db_statement, // TODO bad idea?
        db.operation = db_operation,
        otel.name = "db.operation", // should be <db.operation> <db.name>.<db.sql.table>,
        otel.kind = "CLIENT",
        otel.status_code = tracing::field::Empty,
    )
}

#[tracing::instrument]
pub async fn get_client_handler(
    opts: Option<Query<FilterOptions>>,
    State(data): State<Arc<AppState>>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
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
        .instrument(make_otel_span("SELECT", sql))
        .await;

    if query_result.is_err() {
        let error_response = serde_json::json!({
            "status": "fail",
            "message": "Something bad happened while fetching all client items",
        });
        return Err((StatusCode::INTERNAL_SERVER_ERROR, Json(error_response)));
    }

    let clients = query_result.unwrap();

    let json_response = serde_json::json!({
        "status": "success",
        "results": clients.len(),
        "clients": clients
    });
    Ok(Json(json_response))
}
