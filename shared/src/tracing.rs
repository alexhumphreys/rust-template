use tracing;

pub fn make_otel_db_span(db_operation: &str, db_statement: &str) -> tracing::Span {
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

pub fn make_otel_reqwest_span() -> tracing::Span {
    // NO parsing of statement to extract information, not recommended by Specification and time-consuming
    // warning: providing the statement could leek information
    tracing::trace_span!(
        target: tracing_opentelemetry_instrumentation_sdk::TRACING_TARGET,
        "reqwest request",
        service.name = "api-client",
        otel.name = "should_be_path", // should be <db.operation> <db.name>.<db.sql.table>,
        otel.kind = "CLIENT",
        http.request.method = "GET",
        otel.status_code = tracing::field::Empty,
    )
}
