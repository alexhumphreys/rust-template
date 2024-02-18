use async_trait::async_trait;
use axum::Router as AxumRouter;
use loco_rs::prelude::*;

pub struct OpenFgaInitializer;

#[async_trait]
impl Initializer for OpenFgaInitializer {
    fn name(&self) -> String {
        "axum-prometheus".to_string()
    }

    async fn after_routes(&self, router: AxumRouter, _ctx: &AppContext) -> Result<AxumRouter> {
        todo!()
        // let (prometheus_layer, metric_handle) = axum_prometheus::PrometheusMetricLayer::pair();

        // let router = router
        // .route("/metrics", get(|| async move { metric_handle.render() }))
        // .layer(prometheus_layer);

        // Ok(router)
    }
}
