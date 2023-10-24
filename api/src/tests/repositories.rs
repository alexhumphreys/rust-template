use crate::repositories::Repositories;
use crate::user_repository::MockUserRepo as MockUserRepoImpl;

pub async fn create_repositories_for_test() -> MockRepoImpls {
    MockRepoImpls::new(MockUserRepoImpl::new())
}

#[derive(Debug)]
pub struct MockRepoImpls {
    pub user: MockUserRepoImpl,
}
impl MockRepoImpls {
    pub fn new(mock_user_repo_impl: MockUserRepoImpl) -> Self {
        Self {
            user: mock_user_repo_impl,
        }
    }
}
impl Repositories for MockRepoImpls {
    type UserRepoImpl = MockUserRepoImpl;
    fn user(&self) -> &Self::UserRepoImpl {
        &self.user
    }
}
