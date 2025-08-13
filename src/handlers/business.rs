use std::sync::Arc;

use axum::{
    extract::{ Query, State},
    http::StatusCode,
    response::Json,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    models::{business::BusinessResponse, BusinessCategory},
    repositories::business::BusinessRepository,
    services::overpass::OverpassQuery,
    state::AppState,
};

#[derive(Debug, Deserialize)]
pub struct SyncRequest {
    pub country_code: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct SyncResponse {
    pub businesses_synced: usize,
    pub message: String,
}

#[derive(Debug, Deserialize)]
pub struct ListBusinessesQuery {
    pub limit: Option<i64>,
    pub offset: Option<i64>,
    pub category: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct SearchByRadiusQuery {
    pub latitude: f64,
    pub longitude: f64,
    pub radius_km: i32,
    pub limit: Option<i64>,
}

#[derive(Debug, Deserialize)]
pub struct SearchByRadiusAndCategoryQuery {
    pub latitude: f64,
    pub longitude: f64,
    pub radius_km: i32,
    pub category: BusinessCategory,
    pub limit: Option<i64>,
}

#[derive(Debug, Deserialize)]
pub struct CreateUserBusinessRequest {
    pub id: Uuid,
    pub name: Option<String>,
    pub name_en: Option<String>,
    pub address: Option<String>,
    pub latitude: f64,
    pub longitude: f64,
    pub categories: Vec<BusinessCategory>,
    pub specializations: Option<Vec<String>>,
    pub city: Option<String>,
    pub logo_map_url: Option<String>
}

pub async fn sync_businesses(
    State(state): State<Arc<AppState>>,
    Query(params): Query<SyncRequest>,
) -> Result<Json<SyncResponse>, StatusCode> {
    let country_code = params.country_code.unwrap_or_else(|| "BG".to_string());
    
    tracing::info!("Starting business sync for country: {}", country_code);
    
    // Create Overpass query
    let query = OverpassQuery::car_related_businesses(&country_code);
    
    // Fetch data from Overpass API
    let elements = state.overpass_service
        .execute_query(&query)
        .await
        .map_err(|e| {
            tracing::error!("Failed to fetch data from Overpass API: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    // Create repository
    let repo = BusinessRepository::new(&state.db);
    
    // Sync businesses to database
    let synced_count = repo
        .sync_from_overpass_elements(elements)
        .await
        .map_err(|e| {
            tracing::error!("Failed to sync businesses to database: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    tracing::info!("Successfully synced {} businesses", synced_count);

    Ok(Json(SyncResponse {
        businesses_synced: synced_count,
        message: format!("Successfully synced {} businesses", synced_count),
    }))
}

pub async fn sync_user_business(
    State(state): State<Arc<AppState>>,
    Json(req): Json<CreateUserBusinessRequest>,
) -> Result<Json<BusinessResponse>, StatusCode> {
    let repo = BusinessRepository::new(&state.db);

    // Insert into the search‑engine DB
    let business = repo
        .sync_user_business(req)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(business.to_response()))
}

pub async fn list_businesses(
    State(state): State<Arc<AppState>>,
    Query(params): Query<ListBusinessesQuery>,
) -> Result<Json<Vec<BusinessResponse>>, StatusCode> {
    let repo = BusinessRepository::new(&state.db);
    
    let businesses = repo
        .list_businesses(
            params.limit.unwrap_or(50),
            params.offset.unwrap_or(0),
            params.category.as_deref(),
        )
        .await
        .map_err(|e| {
            tracing::error!("Failed to fetch businesses: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    let business_responses: Vec<BusinessResponse> = businesses
        .into_iter()
        .map(|b| b.to_response())
        .collect();

    Ok(Json(business_responses))
}

pub async fn search_businesses_by_radius(
    State(state): State<Arc<AppState>>,
    Query(params): Query<SearchByRadiusQuery>,
) -> Result<Json<Vec<BusinessResponse>>, StatusCode> {
    let repo = BusinessRepository::new(&state.db);
    
    let businesses = repo
        .get_businesses_by_location(
            params.latitude,
            params.longitude,
            params.radius_km,
            params.limit.unwrap_or(50),
        )
        .await
        .map_err(|e| {
            tracing::error!("Failed to search businesses by radius: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    let business_responses: Vec<BusinessResponse> = businesses
        .into_iter()
        .map(|b| b.to_response())
        .collect();

    Ok(Json(business_responses))
}

pub async fn search_businesses_by_radius_and_category(
    State(state): State<Arc<AppState>>,
    Query(params): Query<SearchByRadiusAndCategoryQuery>,
) -> Result<Json<Vec<BusinessResponse>>, StatusCode> {
    let repo = BusinessRepository::new(&state.db);
    
    let businesses = repo
        .get_businesses_by_location_and_category(
            params.latitude,
            params.longitude,
            params.radius_km,
            &params.category,
            params.limit.unwrap_or(50),
        )
        .await
        .map_err(|e| {
            tracing::error!("Failed to search businesses by radius and category: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    let business_responses: Vec<BusinessResponse> = businesses
        .into_iter()
        .map(|b| b.to_response())
        .collect();

    Ok(Json(business_responses))
}