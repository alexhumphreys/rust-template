use crate::error::Error;
use crate::{client::get_client_by_token, model::ClientModel};
use axum::{
    extract::TypedHeader,
    headers::authorization::{Authorization, Bearer},
    http::Request,
    http::StatusCode,
    middleware::Next,
    response::Response,
};

pub async fn auth<B>(
    // run the `TypedHeader` extractor
    TypedHeader(auth): TypedHeader<Authorization<Bearer>>,
    // you can also add more extractors here but the last
    // extractor must implement `FromRequest` which
    // `Request` does
    request: Request<B>,
    next: Next<B>,
) -> Result<Response, Error> {
    if token_is_valid(auth.token()) {
        let response = next.run(request).await;
        Ok(response)
    } else {
        Err(Error::Unauthorized)
    }
}

fn token_is_valid(token: &str) -> bool {
    tracing::info!("token provided: {}", token);
    return true;
}

pub async fn token_auth<B>(
    // run the `TypedHeader` extractor
    TypedHeader(auth): TypedHeader<Authorization<Bearer>>,
    // you can also add more extractors here but the last
    // extractor must implement `FromRequest` which
    // `Request` does
    mut request: Request<B>,
    next: Next<B>,
) -> Result<Response, Error> {
    match is_valid_client_token(auth.token()).await {
        Ok(client) => {
            request.extensions_mut().insert(client);
            let response = next.run(request).await;
            Ok(response)
        }
        Err(e) => {
            tracing::error!("Error: {}", e);
            Err(Error::Unauthorized)
        }
    }
}

async fn is_valid_client_token(token: &str) -> Result<ClientModel, Error> {
    tracing::info!("token provided: {}", token);
    let client = get_client_by_token(token.to_string(), None).await?;

    Ok(client)
}
