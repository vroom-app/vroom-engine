use std::sync::Arc;
use sqlx::PgPool;
use crate::{config::Config, services::overpass::OverpassService};

#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,
    pub config: Config,
    pub overpass_service: Arc<OverpassService>,
}

impl AppState {
    pub fn new(db: PgPool, config: Config) -> Self {
        let overpass_service = Arc::new(OverpassService::new());
        
        Self {
            db,
            config,
            overpass_service,
        }
    }
}