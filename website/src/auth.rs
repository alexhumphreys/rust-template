use axum::{
    async_trait,
    extract::TypedHeader,
    headers::authorization::{Authorization, Bearer},
    http::Request,
    http::StatusCode,
    middleware::Next,
    response::Response,
};
use axum_session::{Session, SessionConfig, SessionLayer, SessionNullPool, SessionStore};
use axum_session_auth::{
    Auth, AuthConfig, AuthSession, AuthSessionLayer, Authentication, HasPermission, Rights,
};
use serde::{Deserialize, Serialize};
use std::{collections::HashSet, sync::Arc};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: i32,
    pub anonymous: bool,
    pub username: String,
    pub permissions: HashSet<String>,
}

impl Default for User {
    fn default() -> Self {
        let mut permissions = HashSet::new();

        permissions.insert("Category::View".to_owned());

        Self {
            id: 1,
            anonymous: true,
            username: "Guest".into(),
            permissions,
        }
    }
}

// We place our Type within a Arc<> so we can send it across async threads.
type NullPool = Arc<Option<()>>;

#[async_trait]
impl Authentication<User, i64, NullPool> for User {
    async fn load_user(userid: i64, _pool: Option<&NullPool>) -> Result<User, anyhow::Error> {
        if userid == 1 {
            Ok(User::default())
        } else {
            let mut permissions = HashSet::new();

            permissions.insert("Category::View".to_owned());
            permissions.insert("Admin::View".to_owned());

            Ok(User {
                id: 2,
                anonymous: false,
                username: "Test".to_owned(),
                permissions,
            })
        }
    }

    fn is_authenticated(&self) -> bool {
        !self.anonymous
    }

    fn is_active(&self) -> bool {
        !self.anonymous
    }

    fn is_anonymous(&self) -> bool {
        self.anonymous
    }
}

#[async_trait]
impl HasPermission<NullPool> for User {
    async fn has(&self, perm: &str, _pool: &Option<&NullPool>) -> bool {
        self.permissions.contains(perm)
    }
}

pub async fn session_auth<B>(
    // run the `TypedHeader` extractor
    auth: AuthSession<User, i64, SessionNullPool, NullPool>,
    // you can also add more extractors here but the last
    // extractor must implement `FromRequest` which
    // `Request` does
    request: Request<B>,
    next: Next<B>,
) -> Result<Response, StatusCode> {
    todo!()
    /*
    if token_is_valid(auth.token()) {
        let response = next.run(request).await;
        Ok(response)
    } else {
        Err(StatusCode::UNAUTHORIZED)
    }
    */
}

fn token_is_valid(token: &str) -> bool {
    tracing::info!("token provided: {}", token);
    return true;
}
