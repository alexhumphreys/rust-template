//! Run with
//!
//! ```not_rust
//! cargo run -p example-templates
//! ```
mod app_state;
mod auth;
mod auth0;
mod handlers;
mod protected_routes;
mod proxy_routes;
mod public_routes;
mod router;

use shared::startup::{create_server, server_setup};
use std::net::SocketAddr;

#[tokio::main]
async fn main() {
    server_setup();

    let app = create_server(vec![router::routes().await]);

    // run it
    let addr = SocketAddr::from(([0, 0, 0, 0], 8000));
    tracing::info!("listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
