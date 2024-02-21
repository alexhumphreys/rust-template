use async_trait::async_trait;
use axum::Router as AxumRouter;
use loco_rs::prelude::*;
use shared::openfga;

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

#[async_trait]
impl Initializer for OpenFgaInitializer {
    fn name(&self) -> String {
        "openfga".to_string()
    }

    async fn after_routes(&self, router: AxumRouter, _ctx: &AppContext) -> Result<AxumRouter> {
        // create or get store
        let store = create_or_get_store().await?;
        // read model from a file
        let model_string = std::fs::read_to_string("model.pb")?;

        let model = openfga::write_authorization_model(store.id.clone(), model_string, None).await;

        // TODO
        // let router = router
        // .layer(openfga_layer);

        Ok(router)
    }
}
