use std::sync::Arc;
use sqlx::PgPool;
use crate::{
    config::config::Config, 
    domain::services::business_service::BusinessService, 
    infrastructure::{
        database::business_repository_impl::PostgresBusinessRepository, 
        external::overpass::OverpassService
    }
};

#[derive(Clone)]
pub struct AppState {
    pub config: Config,
    pub business_service: Arc<BusinessService>,
}

impl AppState {
    pub fn new(db: PgPool, config: Config) -> Self {
        let overpass_service = Arc::new(OverpassService::new());
        let business_repository = Arc::new(PostgresBusinessRepository::new(db));
        
        // Initialize domain service with dependencies
        let business_service = Arc::new(
            BusinessService::new(business_repository, overpass_service)
        );
        Self {
            config,
            business_service
        }
    }
}