use crate::{
    client_repository::{ClientRepo, ClientRepoImpl},
    db_init,
    user_repository::{UserRepo, UserRepoImpl},
};
use axum::extract::Extension;
use std::sync::Arc;

pub type RepoExt = Extension<Arc<RepoImpls>>;

pub async fn create_repositories() -> RepoImpls {
    let db_pool = Arc::new(db_init::db_connect().await);
    RepoImpls::new(
        UserRepoImpl::new(db_pool.clone()),
        ClientRepoImpl::new(db_pool),
    )
}

#[derive(Debug, Clone)]
pub struct RepoImpls {
    pub user: UserRepoImpl,
    pub client: ClientRepoImpl,
}
impl RepoImpls {
    pub fn new(user_repo_impl: UserRepoImpl, client_repo_impl: ClientRepoImpl) -> Self {
        Self {
            user: user_repo_impl,
            client: client_repo_impl,
        }
    }
}

pub trait Repositories {
    type UserRepoImpl: UserRepo;
    type ClientRepoImpl: ClientRepo;
    fn user(&self) -> &Self::UserRepoImpl;
    fn client(&self) -> &Self::ClientRepoImpl;
}
impl Repositories for RepoImpls {
    type UserRepoImpl = UserRepoImpl;
    type ClientRepoImpl = ClientRepoImpl;
    fn user(&self) -> &Self::UserRepoImpl {
        &self.user
    }
    fn client(&self) -> &Self::ClientRepoImpl {
        &self.client
    }
}
