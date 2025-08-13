use std::collections::HashMap;

use reqwest::Client;
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum OverpassError {
    #[error("HTTP request failed: {0}")]
    Http(#[from] reqwest::Error),
    #[error("JSON parsing failed: {0}")]
    Json(#[from] serde_json::Error),
    #[error("Query execution failed: {0}")]
    Query(String),
}

#[derive(Debug, Clone)]
pub struct OverpassService {
    client: Client,
    base_url: String,
}

#[derive(Debug, Clone)]
pub struct OverpassQuery {
    pub query: String,
    pub timeout: u64,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct OverpassResponse {
    pub elements: Vec<OverpassElement>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct OverpassElement {
    #[serde(rename = "type")]
    pub element_type: String,
    pub id: i64,
    pub lat: Option<f64>,
    pub lon: Option<f64>,
    pub tags: Option<HashMap<String, String>>,
}

impl OverpassService {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
            base_url: "https://overpass-api.de/api/interpreter".to_string(),
        }
    }

    pub fn with_base_url(base_url: String) -> Self {
        Self {
            client: Client::new(),
            base_url,
        }
    }

    pub async fn execute_query(
        &self,
        query: &OverpassQuery,
    ) -> Result<Vec<OverpassElement>, OverpassError> {
        tracing::debug!("Executing Overpass query: {}", query.query);

        let response = self
            .client
            .post(&self.base_url)
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(format!("data={}", urlencoding::encode(&query.query)))
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(OverpassError::Query(format!(
                "HTTP {}: {}",
                response.status(),
                response.text().await.unwrap_or_default()
            )));
        }

        let overpass_response: OverpassResponse = response.json().await?;
        
        tracing::info!("Received {} elements from Overpass API", overpass_response.elements.len());
        
        Ok(overpass_response.elements)
    }
}

impl OverpassQuery {
    pub fn car_related_businesses(country_code: &str) -> Self {
        let query = format!(
            r#"[out:json][timeout:50];
area["ISO3166-1"="{}"][admin_level=2]->.searchArea;

// Nodes for car-related amenities
(
  node["amenity"="car_wash"](area.searchArea);
  node["amenity"="fuel"](area.searchArea);
  node["amenity"="charging_station"](area.searchArea);
  node["amenity"="car_rental"](area.searchArea);
  node["amenity"="parking"](area.searchArea);
  node["shop"="car_repair"](area.searchArea);
  node["shop"="car"](area.searchArea);
  node["shop"="car_parts"](area.searchArea);
  node["shop"="tyres"](area.searchArea);
  node["craft"="car_repair"](area.searchArea);
  node["service"="vehicle_inspection"](area.searchArea);
);
out body;
>;
out skel qt;"#,
            country_code
        );

        Self {
            query,
            timeout: 50,
        }
    }

    pub fn custom(query: String, timeout: u64) -> Self {
        Self { query, timeout }
    }
}