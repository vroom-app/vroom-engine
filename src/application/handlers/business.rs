use axum::{
    extract::{Query, State},
    http::StatusCode,
    Json,
};
use bigdecimal::BigDecimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use std::sync::Arc;
use crate::{application::state::AppState, domain::entities::{business::BusinessResponse, category::BusinessCategory}};

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
    pub logo_map_url: Option<String>,
    pub average_reviews: Option<BigDecimal>,
    pub review_count: Option<i32>, 
}

pub async fn sync_businesses(
    State(state): State<Arc<AppState>>,
    Query(params): Query<SyncRequest>,
) -> Result<Json<SyncResponse>, StatusCode> {
    let country_code = params.country_code.unwrap_or_else(|| "BG".to_string());
    
    let synced_count = state.business_service
        .sync_businesses(&country_code)
        .await
        .map_err(|e| {
            tracing::error!("Failed to sync businesses: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    Ok(Json(SyncResponse {
        businesses_synced: synced_count,
        message: format!("Successfully synced {} businesses", synced_count),
    }))
}

pub async fn sync_user_business(
    State(state): State<Arc<AppState>>,
    Json(req): Json<CreateUserBusinessRequest>,
) -> Result<Json<BusinessResponse>, StatusCode> {
    let business = state.business_service
        .create_user_business(req)
        .await
        .map_err(|e| {
            tracing::error!("Failed to create user business: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    Ok(Json(business.to_response()))
}

pub async fn search_businesses_by_radius_and_category(
    State(state): State<Arc<AppState>>,
    Query(params): Query<SearchByRadiusAndCategoryQuery>,
) -> Result<Json<Vec<BusinessResponse>>, StatusCode> {
    let businesses = state.business_service
        .search_businesses_by_radius_and_category(
            params.latitude,
            params.longitude,
            params.radius_km,
            &params.category,
            params.limit,
        )
        .await
        .map_err(|e| {
            tracing::error!("Failed to search businesses: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    let business_responses: Vec<BusinessResponse> = businesses
        .into_iter()
        .map(|b| b.to_response())
        .collect();

    Ok(Json(business_responses))
}