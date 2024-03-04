use async_trait::async_trait;
use axum::Router as AxumRouter;
use loco_rs::prelude::*;
use shared::openfga::{self, WriteAuthorizationModelResponse};

pub struct OpenFgaInitializer;

async fn create_or_get_store() -> Result<openfga::CreateDataStoreResponse> {
    let store_name = "foobar2".to_string();
    let res = openfga::create_data_store(
        openfga::CreateDataStoreSchema {
            name: store_name.clone(),
        },
        None,
    )
    .await;

    let store = res.unwrap();
    Ok(store)
}

async fn read_model_from_file() -> Result<String> {
    let model_string = std::fs::read_to_string("model.json")?;
    Ok(model_string)
}

async fn write_model(store_id: String, model: String) -> Result<WriteAuthorizationModelResponse> {
    let model = openfga::write_authorization_model(store_id, model, None).await;
    match model {
        Ok(model) => {
            println!("Model written successfully");
            Ok(model)
        }
        Err(e) => {
            println!("Error writing model: {:?}", e);
            Err(loco_rs::Error::NotFound)
        }
    }
}

async fn initialize_model() -> Result<(WriteAuthorizationModelResponse)> {
    // create or get store
    let store = create_or_get_store().await?;
    // read model from a file
    let model_string = read_model_from_file().await?;

    let model = write_model(store.id.clone(), model_string).await?;
    Ok(model)
}

#[async_trait]
impl Initializer for OpenFgaInitializer {
    fn name(&self) -> String {
        "openfga".to_string()
    }

    async fn after_routes(&self, router: AxumRouter, _ctx: &AppContext) -> Result<AxumRouter> {
        let res = initialize_model();

        // TODO
        // let router = router
        // .layer(openfga_layer);

        Ok(router)
    }
}

#[cfg(test)]
mod tests {
    use crate::initializers::openfga::*;
    use shared::openfga::*;

    #[tokio::test]
    async fn test_initializer() {
        let res = initialize_model().await;

        println!("{:?}", res);
        assert_eq!(false, true);
    }
}
