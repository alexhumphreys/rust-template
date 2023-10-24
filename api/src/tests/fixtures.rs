use shared::model::UserTransportModel;
use uuid::Uuid;

#[allow(dead_code)]
pub fn user_fixture(id: Uuid) -> UserTransportModel {
    UserTransportModel {
        id: id.clone(),
        name: String::from("taro"),
    }
}
