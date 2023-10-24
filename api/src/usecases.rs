use crate::repositories::Repositories;
use crate::user_repository::UserRepo;
use shared::{error::Error, model::UserTransportModel};
use std::sync::Arc;
use uuid::Uuid;

pub async fn view<R: Repositories>(
    repo: Arc<R>,
    user_id: Uuid,
) -> Result<UserTransportModel, Error> {
    let user = repo.user().get_user(user_id).await?;
    Ok(user)
}
/*
pub async fn add<R: Repositories>(repo: Arc<R>, new_user: &NewUser) -> Result<UserId> {
    let user_id = repo.user().add(&new_user).await?;
    Ok(user_id)
}
*/

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests::{fixtures::user_fixture, repositories::create_repositories_for_test};

    #[tokio::test]
    async fn test_get_user() {
        let user_id = Uuid::parse_str("36f9424f-f929-4c78-a28f-6f6c9fcc93b4").unwrap();
        let user_id_cloned = user_id.clone();

        let mut mock_repo_impl = create_repositories_for_test().await;

        mock_repo_impl
            .user
            .expect_get_user()
            .returning(move |_| Ok(user_fixture(user_id_cloned)));
        let user = view(Arc::new(mock_repo_impl), user_id).await.unwrap();
        assert_eq!(user, user_fixture(user_id));
    }
}
