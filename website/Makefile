run:
	OTEL_SERVICE_NAME=website \
	OTEL_LOG_LEVEL=info \
	OTEL_EXPORTER_OTLP_TRACES_ENDPOINT=http://localhost:4317 \
	RUST_LOG=info,axum_tracing_opentelemetry=info,otel=info \
	cargo run
