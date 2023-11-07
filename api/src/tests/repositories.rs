use crate::client_repository::MockClientRepo as MockClientRepoImpl;
use crate::repositories::Repositories;
use crate::user_repository::MockUserRepo as MockUserRepoImpl;

pub async fn create_repositories_for_test() -> MockRepoImpls {
    MockRepoImpls::new(MockUserRepoImpl::new(), MockClientRepoImpl::new())
}

#[derive(Debug)]
pub struct MockRepoImpls {
    pub user: MockUserRepoImpl,
    pub client: MockClientRepoImpl,
}
impl MockRepoImpls {
    pub fn new(
        mock_user_repo_impl: MockUserRepoImpl,
        mock_client_repo_impl: MockClientRepoImpl,
    ) -> Self {
        Self {
            user: mock_user_repo_impl,
            client: mock_client_repo_impl,
        }
    }
}
impl Repositories for MockRepoImpls {
    type UserRepoImpl = MockUserRepoImpl;
    type ClientRepoImpl = MockClientRepoImpl;
    fn user(&self) -> &Self::UserRepoImpl {
        &self.user
    }
    fn client(&self) -> &Self::ClientRepoImpl {
        &self.client
    }
}
