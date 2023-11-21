mod app_state;
mod client_repository;
mod db;
mod db_init;
mod handler;
mod repositories;
mod router;
#[cfg(test)]
mod tests;
mod usecases;
mod user_repository;

use shared::startup::{create_server, server_setup};
use std::net::SocketAddr;

#[tokio::main]
async fn main() {
    server_setup();

    let app = create_server(vec![router::routes().await]);

    // run it
    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    tracing::info!("listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}
