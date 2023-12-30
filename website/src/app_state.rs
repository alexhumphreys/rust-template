use fluent_templates::{ArcLoader, FluentLoader};
use handlebars::Handlebars;
use std::sync::Arc;

#[derive(Debug)]
pub struct AppState {
    pub handlebars: Handlebars<'static>,
}

pub async fn create_app_state() -> Arc<AppState> {
    let arc = ArcLoader::builder("locales", unic_langid::langid!("en-US"))
        .shared_resources(Some(&["./locales/core.ftl".into()]))
        .customize(|bundle| bundle.set_use_isolating(false))
        .build()
        .unwrap();

    let mut handlebars = handlebars::Handlebars::new();
    handlebars.register_helper("fluent", Box::new(FluentLoader::new(arc)));
    handlebars
        .register_templates_directory(".hbs", "handlebars/")
        .unwrap(); // TODO better error handling
    let app_state = Arc::new(AppState { handlebars });
    app_state
}
