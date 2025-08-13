use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

use crate::models::category::{BusinessCategory};

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Business {
    pub id: Uuid,
    pub osm_id: Option<i64>,
    pub name: Option<String>,
    pub name_en: Option<String>,
    pub address: Option<String>,
    pub latitude: f64,
    pub longitude: f64,
    pub categories: Vec<BusinessCategory>,
    pub specializations: Option<Vec<String>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub logo_map_url: Option<String>,
    pub is_registered: bool,
    pub city: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BusinessInsert {
    pub osm_id: i64,
    pub name: Option<String>,
    pub name_en: Option<String>,
    pub address: Option<String>,
    pub latitude: f64,
    pub longitude: f64,
    pub categories: Vec<BusinessCategory>,
    pub city: Option<String>,
}

#[allow(non_snake_case)]
#[derive(Debug, Serialize, Deserialize)]
pub struct BusinessResponse {
    pub id: Uuid,
    pub name: Option<String>,
    pub location: Location,
    pub categories: Vec<String>,
    pub specializations: Option<Vec<String>>,
    pub media: Media,
    pub isRegistered: bool,
    pub rating: Rating,
}

#[allow(non_snake_case)]
#[derive(Debug, Serialize, Deserialize)]
pub struct Media {
    pub mapLogo: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Location {
    pub address: Option<String>,
    pub city: Option<String>,
    pub latitude: f64,
    pub longitude: f64,
}

#[allow(non_snake_case)]
#[derive(Debug, Serialize, Deserialize)]
pub struct Rating {
    pub averageReviews: Option<f32>,
    pub numReviews: Option<u32>,
}

impl BusinessInsert {
    pub fn from_osm_element(
        osm_id: i64,
        lat: f64,
        lon: f64,
        tags: std::collections::HashMap<String, String>,
    ) -> Self {
        let categories = BusinessCategory::from_osm_tags(&tags);

        Self {
            osm_id,
            name: tags.get("name").cloned(),
            name_en: tags.get("name:en").cloned(),
            address: Self::build_address(&tags),
            latitude: lat,
            longitude: lon,
            categories,
            city: tags.get("addr:city").or_else(|| tags.get("city")).cloned(),
        }
    }

    pub async fn upsert(&self, pool: &sqlx::PgPool) -> Result<Uuid, sqlx::Error> {
        let result = sqlx::query!(
            r#"
            INSERT INTO search.businesses (
                osm_id, name, name_en, address, location,
                categories, city
            )
            VALUES (
                $1, $2, $3, $4, ST_SetSRID(ST_MakePoint($5, $6), 4326),
                $7::search.business_category[], $8
            )
            ON CONFLICT (osm_id)
            DO UPDATE SET
                name            = EXCLUDED.name,
                name_en         = EXCLUDED.name_en,
                address         = EXCLUDED.address,
                location        = EXCLUDED.location,
                categories      = EXCLUDED.categories,
                updated_at      = NOW(),
                city            = EXCLUDED.city
            WHERE search.businesses.is_registered = FALSE
            RETURNING id
            "#,
            self.osm_id,
            self.name,
            self.name_en,
            self.address,
            self.longitude,
            self.latitude,
            &self.categories as &[BusinessCategory],
            self.city,
        )
        .fetch_one(pool)
        .await?;

        Ok(result.id)
    }

    fn build_address(tags: &std::collections::HashMap<String, String>) -> Option<String> {
        let mut parts = Vec::new();
        if let Some(street) = tags.get("addr:street") {
            if let Some(number) = tags.get("addr:housenumber") {
                parts.push(format!("{} {}", street, number));
            } else {
                parts.push(street.clone());
            }
        }
        for key in &[
            "addr:city", "addr:postcode", "addr:country",
        ] {
            if let Some(v) = tags.get(*key) {
                parts.push(v.clone());
            }
        }
        (!parts.is_empty()).then(|| parts.join(", "))
    }
}

impl From<Business> for BusinessResponse {
    fn from(business: Business) -> Self {
        Self {
            id: business.id,
            name: business.name,
            location: Location { 
                city: business.city,
                address: business.address,
                latitude: business.latitude, 
                longitude: business.longitude
            },
            categories: business.categories
                .iter()
                .map(|c| c.display_name().to_string())
                .collect(),
            specializations: business.specializations,
            media: Media { 
                mapLogo: business.logo_map_url, 
            },
            rating: Rating { 
                averageReviews: Some(5.0), 
                numReviews: Some(1) 
            },
            isRegistered: business.is_registered,
        }
    }
}

// Optional: Business filtering and search structures
#[derive(Debug, Deserialize)]
pub struct BusinessFilter {
    pub categories: Option<Vec<String>>,
    pub has_contact: Option<bool>,
    pub is_registered: Option<bool>,
    pub search_term: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct LocationFilter {
    pub latitude: f64,
    pub longitude: f64,
    pub radius_km: f64,
}

#[derive(Debug, Serialize)]
pub struct BusinessListResponse {
    pub businesses: Vec<BusinessResponse>,
    pub total_count: i64,
    pub page: i64,
    pub per_page: i64,
}

impl Business {
    /// Convert to API response format
    pub fn to_response(self) -> BusinessResponse {
        self.into()
    }

    /// Check if business matches search term
    pub fn matches_search_term(&self, term: &str) -> bool {
        let term_lower = term.to_lowercase();
        
        self.name.as_ref().map_or(false, |n| n.to_lowercase().contains(&term_lower)) ||
        self.address.as_ref().map_or(false, |a| a.to_lowercase().contains(&term_lower)) ||
        self.categories.iter().any(|c| c.display_name().to_lowercase().contains(&term_lower))
    }
}