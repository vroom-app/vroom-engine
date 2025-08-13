use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    handlers::business::CreateUserBusinessRequest, models::{business::{Business, BusinessInsert}, category::{BusinessCategory}}, services::overpass::OverpassElement
};

pub struct BusinessRepository<'a> {
    pool: &'a PgPool,
}

impl<'a> BusinessRepository<'a> {
    pub fn new(pool: &'a PgPool) -> Self {
        Self { pool }
    }

    pub async fn sync_from_overpass_elements(
        &self,
        elements: Vec<OverpassElement>,
    ) -> Result<usize, sqlx::Error> {
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

            match business_insert.upsert(self.pool).await {
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

    pub async fn sync_user_business(
        &self,
        req: CreateUserBusinessRequest,
    ) -> Result<Business, sqlx::Error> {
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
        .execute(self.pool)
        .await?;

        self.get_business_by_id(id)
            .await?
            .ok_or(sqlx::Error::RowNotFound)
    }

    pub async fn list_businesses(
        &self,
        limit: i64,
        offset: i64,
        category_filter: Option<&str>,
    ) -> Result<Vec<Business>, sqlx::Error> {
        match category_filter {
            Some(category) => {
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
                        city
                    FROM search.businesses
                    WHERE $1 = ANY(categories::text[])
                    ORDER BY created_at DESC
                    LIMIT $2 OFFSET $3
                    "#,
                    category,
                    limit,
                    offset
                )
                .fetch_all(self.pool)
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
            None => {
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
                        city
                    FROM search.businesses
                    ORDER BY created_at DESC
                    LIMIT $1 OFFSET $2
                    "#,
                    limit,
                    offset
                )
                .fetch_all(self.pool)
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
    }

    pub async fn get_business_by_id(&self, id: Uuid) -> Result<Option<Business>, sqlx::Error> {
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
        .fetch_optional(self.pool)
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

    pub async fn get_businesses_by_location(
        &self,
        lat: f64,
        lon: f64,
        radius_km: i32,
        limit: i64,
    ) -> Result<Vec<Business>, sqlx::Error> {
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
            ORDER BY ST_Distance(
                location,
                ST_SetSRID(ST_MakePoint($2, $1), 4326)::geography
            )
            LIMIT $4
            "#,
            lat,
            lon,
            radius_km,
            limit
        )
        .fetch_all(self.pool)
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

    pub async fn delete_business(&self, id: Uuid) -> Result<bool, sqlx::Error> {
        let result = sqlx::query!(
            "DELETE FROM search.businesses WHERE id = $1 AND is_registered = false",
            id
        )
        .execute(self.pool)
        .await?;

        Ok(result.rows_affected() > 0)
    }

    pub async fn get_businesses_by_location_and_category(
        &self,
        lat: f64,
        lon: f64,
        radius_km: i32,
        category: &BusinessCategory,
        limit: i64,
    ) -> Result<Vec<Business>, sqlx::Error> {
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
        .fetch_all(self.pool)
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