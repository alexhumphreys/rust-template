FROM rust:1.70.0 AS chef 
RUN cargo install cargo-chef 
WORKDIR build

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

# Build the application
FROM chef AS builder 
COPY --from=planner /build/recipe.json recipe.json
# Build dependencies - this is the caching Docker layer!
RUN cargo chef cook --release --recipe-path recipe.json
# Build application
COPY . .
ARG SQLX_OFFLINE=true
RUN cargo build --release

# create the base runtime docker image
FROM debian:11-slim AS runtime
ENV APP=/usr/src/app
RUN apt-get update \
    && apt-get install -y ca-certificates tzdata openssl libssl-dev \
    && rm -rf /var/lib/apt/lists/*
ENV TZ=Etc/UTC \
    APP_USER=appuser
RUN groupadd $APP_USER \
    && useradd -g $APP_USER $APP_USER \
    && mkdir -p $APP

# Run the API
FROM runtime AS api
COPY --from=builder /build/target/release/api-server ${APP}/api-server
EXPOSE 3000
RUN chown -R $APP_USER:$APP_USER $APP
USER $APP_USER
WORKDIR ${APP}
CMD ["./api-server"]

# Run the website
FROM runtime AS website
COPY --from=builder /build/target/release/website ${APP}/website
EXPOSE 8000
RUN chown -R $APP_USER:$APP_USER $APP
USER $APP_USER
WORKDIR ${APP}
CMD ["./website"]
