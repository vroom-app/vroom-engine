use std::sync::Arc;
use crate::application::handlers::business::CreateUserBusinessRequest;
use crate::domain::entities::business::Business;
use crate::domain::entities::category::BusinessCategory;
use crate::domain::repositories::business_repository::BusinessRepository;
use crate::infrastructure::external::overpass::{OverpassQuery, OverpassService};
use crate::shared::error::{AppError, Result};

pub struct BusinessService {
    business_repository: Arc<dyn BusinessRepository>,
    overpass_service: Arc<OverpassService>,
}

impl BusinessService {
    pub fn new(
        business_repository: Arc<dyn BusinessRepository>,
        overpass_service: Arc<OverpassService>,
    ) -> Self {
        Self {
            business_repository,
            overpass_service,
        }
    }

    pub async fn sync_businesses(&self, country_code: &str) -> Result<usize> {
        tracing::info!("Starting business sync for country: {}", country_code);
        
        // Create Overpass query
        let query = OverpassQuery::car_related_businesses(country_code);
        
        // Fetch data from Overpass API
        let elements = self.overpass_service
            .execute_query(&query)
            .await
            .map_err(|e| AppError::OsmParsing(format!("Error fetching data from overpass: {}", e)))?;

        // Sync businesses to database
        let synced_count = self.business_repository
            .sync_from_overpass_elements(elements)
            .await?;

        tracing::info!("Successfully synced {} businesses", synced_count);
        Ok(synced_count)
    }

    pub async fn create_user_business(&self, req: CreateUserBusinessRequest) -> Result<Business> {
        self.business_repository
            .sync_user_business(req)
            .await
    }

    pub async fn search_businesses_by_radius_and_category(
        &self,
        latitude: f64,
        longitude: f64,
        radius_km: i32,
        category: &BusinessCategory,
        limit: Option<i64>,
    ) -> Result<Vec<Business>> {
        self.business_repository
            .get_businesses_by_location_and_category(
                latitude,
                longitude,
                radius_km,
                category,
                limit.unwrap_or(50),
            )
            .await
    }
}