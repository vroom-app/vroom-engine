use axum::async_trait;
use uuid::Uuid;
use crate::application::handlers::business::CreateUserBusinessRequest;
use crate::domain::entities::business::{Business};
use crate::infrastructure::external::overpass::OverpassElement;
use crate::shared::error::Result;
use crate::domain::entities::category::BusinessCategory;

#[async_trait]
pub trait BusinessRepository: Send + Sync {
    /// Sync businesses from Overpass elements to the database.
    /// This method processes each element, checks if it is car-related,
    /// and inserts or updates the business in the database.
    async fn sync_from_overpass_elements(
        &self,
        elements: Vec<OverpassElement>,
    ) -> Result<usize>;

    /// Sync a user-created business to the database.
    /// This method takes a request containing business details and inserts it into the database.
    async fn sync_user_business(
        &self,
        req: CreateUserBusinessRequest,
    ) -> Result<Business>;

    /// Get a business by its ID.
    /// This method retrieves a business from the database using its unique identifier.
    async fn get_business_by_id(&self, id: Uuid) -> Result<Option<Business>>;

    /// Search for businesses within a specified radius and category.
    /// This method retrieves businesses that are within a certain distance from a given point
    async fn get_businesses_by_location_and_category(
        &self,
        latitude: f64,
        longitude: f64,
        radius_km: i32,
        category: &BusinessCategory,
        limit: i64,
    ) -> Result<Vec<Business>>;
}