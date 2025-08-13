use axum::async_trait;
use sqlx::PgPool;
use uuid::Uuid;
use crate::application::handlers::business::CreateUserBusinessRequest;
use crate::domain::entities::category::BusinessCategory;
use crate::domain::repositories::business_repository::BusinessRepository;
use crate::domain::entities::business::{Business, BusinessInsert};
use crate::infrastructure::external::overpass::OverpassElement;
use crate::shared::error::{Result};

pub struct PostgresBusinessRepository {
    pool: PgPool,
}

impl PostgresBusinessRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl BusinessRepository for PostgresBusinessRepository {
    async fn sync_from_overpass_elements(
        &self,
        elements: Vec<OverpassElement>,
    ) -> Result<usize> {
        let mut synced_count = 0;

        for element in elements {
            // Skip elements without coordinates or tags
            let (lat, lon) = match (element.lat, element.lon) {
                (Some(lat), Some(lon)) => (lat, lon),
                _ => {
                    tracing::debug!("Skipping element {} without coordinates", element.id);
                    continue;
                }
            };

            let tags = element.tags.unwrap_or_default();
            
            // Skip if not car-related
            if !BusinessCategory::is_car_related_osm_element(&tags) {
                continue;
            }

            let business_insert = BusinessInsert::from_osm_element(
                element.id,
                lat,
                lon,
                tags,
            );

            match self.upsert_business(&business_insert).await {
                Ok(_) => {
                    synced_count += 1;
                    if synced_count % 100 == 0 {
                        tracing::info!("Synced {} businesses so far", synced_count);
                    }
                }
                Err(e) => {
                    tracing::error!("Failed to upsert business {}: {}", element.id, e);
                }
            }
        }

        Ok(synced_count)
    }

    async fn sync_user_business(
        &self,
        req: CreateUserBusinessRequest,
    ) -> Result<Business> {
        let id = req.id;
        let specialization = &req.specializations.unwrap_or(Vec::new());

        sqlx::query!(
            r#"
            INSERT INTO search.businesses (
            id, name, name_en, address, location, categories, specializations, is_registered, city, logo_map_url
            ) VALUES (
            $1, $2, $3, $4,
            ST_SetSRID(ST_MakePoint($5, $6), 4326),
            $7::search.business_category[], $8, TRUE, $9, $10
            )
            ON CONFLICT (id) DO UPDATE SET
            name = EXCLUDED.name,
            name_en = EXCLUDED.name_en,
            address = EXCLUDED.address,
            location = EXCLUDED.location,
            categories = EXCLUDED.categories,
            specializations = EXCLUDED.specializations,
            is_registered = TRUE,
            city = EXCLUDED.city,
            logo_map_url = EXCLUDED.logo_map_url
            "#,
            id,
            req.name,
            req.name_en,
            req.address,
            req.longitude,
            req.latitude,
            &req.categories as &[BusinessCategory],
            specialization as &[String],
            req.city,
            req.logo_map_url
        )
        .execute(&self.pool)
        .await?;

        self.get_business_by_id(id)
            .await?
            .ok_or_else(|| sqlx::Error::RowNotFound.into())
    }

    async fn get_business_by_id(&self, id: Uuid) -> Result<Option<Business>> {
        let business = sqlx::query!(
            r#"
            SELECT 
                id,
                osm_id,
                name,
                name_en,
                address,
                ST_Y(location) as latitude,
                ST_X(location) as longitude,
                categories as "categories!: Vec<BusinessCategory>",
                specializations as "specializations!: Vec<String>",
                created_at,
                updated_at,
                logo_map_url,
                is_registered,
                city
            FROM search.businesses
            WHERE id = $1
            "#,
            id
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(business.map(|row| Business {
            id: row.id,
            osm_id: row.osm_id,
            name: row.name,
            name_en: row.name_en,
            address: row.address,
            latitude: row.latitude.unwrap_or(0.0),
            longitude: row.longitude.unwrap_or(0.0),
            categories: row.categories,
            specializations: Some(row.specializations),
            created_at: row.created_at.expect("created_at should never be null"),
            updated_at: row.updated_at.expect("updated_at should never be null"),
            logo_map_url: row.logo_map_url,
            is_registered: row.is_registered.unwrap_or(false),
            city: row.city
        }))
    }

    async fn get_businesses_by_location_and_category(
        &self,
        lat: f64,
        lon: f64,
        radius_km: i32,
        category: &BusinessCategory,
        limit: i64,
    ) -> Result<Vec<Business>> {
        let category_str = category.to_string();

        let businesses = sqlx::query!(
            r#"
            SELECT 
                id,
                osm_id,
                name,
                name_en,
                address,
                ST_Y(location) as latitude,
                ST_X(location) as longitude,
                categories as "categories!: Vec<BusinessCategory>",
                specializations as "specializations!: Vec<String>",
                created_at,
                updated_at,
                logo_map_url,
                is_registered,
                city,
                ST_Distance(
                    location,
                    ST_SetSRID(ST_MakePoint($2, $1), 4326)::geography
                ) as distance_meters
            FROM search.businesses
            WHERE ST_DWithin(
                location,
                ST_SetSRID(ST_MakePoint($2, $1), 4326)::geography,
                $3 * 1000
            )
            AND $4 = ANY(categories::text[])
            ORDER BY ST_Distance(
                location,
                ST_SetSRID(ST_MakePoint($2, $1), 4326)::geography
            )
            LIMIT $5
            "#,
            lat,
            lon,
            radius_km,
            category_str,
            limit
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(businesses.into_iter().map(|row| Business {
            id: row.id,
            osm_id: row.osm_id,
            name: row.name,
            name_en: row.name_en,
            address: row.address,
            latitude: row.latitude.unwrap_or(0.0),
            longitude: row.longitude.unwrap_or(0.0),
            categories: row.categories,
            specializations: Some(row.specializations),
            created_at: row.created_at.expect("created_at should never be null"),
            updated_at: row.updated_at.expect("updated_at should never be null"),
            logo_map_url: row.logo_map_url,
            is_registered: row.is_registered.unwrap_or(false),
            city: row.city
        }).collect())
    }
}

impl PostgresBusinessRepository {
    async fn upsert_business(&self, business: &BusinessInsert) -> Result<Uuid> {
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
            business.osm_id,
            business.name,
            business.name_en,
            business.address,
            business.longitude,
            business.latitude,
            &business.categories as &[BusinessCategory],
            business.city,
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(result.id)
    }
}