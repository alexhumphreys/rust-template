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
use shared::client;
use std::{collections::HashSet, sync::Arc};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: Uuid,
    pub anonymous: bool,
    pub username: String,
    pub permissions: HashSet<String>,
}

impl Default for User {
    fn default() -> Self {
        let mut permissions = HashSet::new();

        permissions.insert("Category::View".to_owned());

        Self {
            id: uuid::Uuid::new_v4(),
            anonymous: true,
            username: "Guest".into(),
            permissions,
        }
    }
}

// We place our Type within a Arc<> so we can send it across async threads.
type NullPool = Arc<Option<()>>;

#[async_trait]
impl Authentication<User, Option<Uuid>, NullPool> for User {
    async fn load_user(
        userid: Option<Uuid>,
        _pool: Option<&NullPool>,
    ) -> Result<User, anyhow::Error> {
        let user = match userid {
            Some(id) => {
                tracing::info!("Looking up user {}", id);
                match client::get_user(id, None).await {
                    Ok(user) => user,
                    Err(e) => {
                        tracing::error!("Error: {}", e);
                        return Ok(User::default());
                    }
                }
            }
            None => return Ok(User::default()),
        };
        tracing::info!("found user {}", user.id);
        let mut permissions = HashSet::new();

        permissions.insert("Category::View".to_owned());
        permissions.insert("Admin::View".to_owned());

        Ok(User {
            id: user.id,
            anonymous: false,
            username: user.name,
            permissions,
        })
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
    auth: AuthSession<User, Option<Uuid>, SessionNullPool, NullPool>,
    // you can also add more extractors here but the last
    // extractor must implement `FromRequest` which
    // `Request` does
    request: Request<B>,
    next: Next<B>,
) -> Result<Response, StatusCode> {
    if auth.is_anonymous() {
        Err(StatusCode::UNAUTHORIZED)
    } else {
        let response = next.run(request).await;
        Ok(response)
    }
}
