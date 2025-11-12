use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use strum_macros::Display;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, sqlx::Type, Display)]
#[sqlx(type_name = "search._business_category", rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum BusinessCategory {
    CarWash,
    Mobile,
    CarRepair,
    Parking,
    GasStation,
    ElectricVehicleChargingStation,
    CarDealer,
    CarRental,
    DetailingStudio,
    RimsShop,
    Tuning,
    TireShop,
    CarInspectionStation,
}

impl BusinessCategory {
    /// Extract business categories from OSM tags
    pub fn from_osm_tags(tags: &HashMap<String, String>) -> Vec<Self> {
        let mut categories = Vec::new();
        
        // Check amenity tags
        if let Some(amenity) = tags.get("amenity") {
            if let Some(category) = Self::from_amenity_tag(amenity) {
                categories.push(category);
            }
        }

        // Check shop tags
        if let Some(shop) = tags.get("shop") {
            if let Some(category) = Self::from_shop_tag(shop) {
                categories.push(category);
            }
        }

        // Check craft tags
        if let Some(craft) = tags.get("craft") {
            if let Some(category) = Self::from_craft_tag(craft) {
                categories.push(category);
            }
        }

        // Check service tags
        if let Some(service) = tags.get("service") {
            if let Some(category) = Self::from_service_tag(service) {
                categories.push(category);
            }
        }

        // Check automotive tags
        if let Some(automotive) = tags.get("automotive") {
            if let Some(category) = Self::from_automotive_tag(automotive) {
                categories.push(category);
            }
        }

        // Check for car wash by name or specific keys
        if Self::is_car_wash_by_name_or_key(tags) {
            categories.push(Self::CarWash);
        }

        // Remove duplicates and sort
        categories.sort();
        categories.dedup();
        
        // Log interesting findings (excluding common ones like parking/gas stations)
        if !categories.is_empty() && Self::should_log_categories(&categories) {
            tracing::debug!(
                "Found car-related business with categories: {:?}, key tags: {:?}", 
                categories, 
                Self::extract_key_tags(tags)
            );
        }
        
        categories
    }

    /// Check if OSM element is car-related
    pub fn is_car_related_osm_element(tags: &HashMap<String, String>) -> bool {
        !Self::from_osm_tags(tags).is_empty()
    }

    // Private helper methods for cleaner organization
    fn from_amenity_tag(amenity: &str) -> Option<Self> {
        match amenity {
            "fuel" => Some(Self::GasStation),
            "charging_station" => Some(Self::ElectricVehicleChargingStation),
            "car_wash" => Some(Self::CarWash),
            "car_rental" => Some(Self::CarRental),
            "parking" | "parking_space" => Some(Self::Parking),
            _ => None,
        }
    }

    fn from_shop_tag(shop: &str) -> Option<Self> {
        match shop {
            "car_repair" | "car_parts" => Some(Self::CarRepair),
            "car" => Some(Self::CarDealer),
            "tyres" => Some(Self::TireShop),
            "wheels" => Some(Self::RimsShop),
            _ => None,
        }
    }

    fn from_craft_tag(craft: &str) -> Option<Self> {
        match craft {
            "car_repair" | "automotive" => Some(Self::CarRepair),
            _ => None,
        }
    }

    fn from_service_tag(service: &str) -> Option<Self> {
        match service {
            "vehicle_inspection" => Some(Self::CarInspectionStation),
            "car_wash" => Some(Self::CarWash),
            _ => None,
        }
    }

    fn from_automotive_tag(automotive: &str) -> Option<Self> {
        match automotive {
            "car_wash" => Some(Self::CarWash),
            "car_repair" => Some(Self::CarRepair),
            "fuel" => Some(Self::GasStation),
            _ => None,
        }
    }

    fn is_car_wash_by_name_or_key(tags: &HashMap<String, String>) -> bool {
        tags.contains_key("car_wash") ||
        tags.get("name").map_or(false, |n| {
            let name_lower = n.to_lowercase();
            name_lower.contains("car wash") || name_lower.contains("автомивка")
        })
    }

    fn should_log_categories(categories: &[Self]) -> bool {
        !categories.iter().all(|c| matches!(c, Self::Parking | Self::GasStation))
    }

    fn extract_key_tags(tags: &HashMap<String, String>) -> HashMap<String, String> {
        const INTERESTING_KEYS: &[&str] = &[
            "amenity", "shop", "craft", "service", "automotive", "name", "brand"
        ];
        
        tags.iter()
            .filter(|(k, _)| INTERESTING_KEYS.contains(&k.as_str()))
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect()
    }

    /// Get display name for the category
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::CarWash => "CarWash",
            Self::Mobile => "Mobile",
            Self::CarRepair => "CarRepair",
            Self::Parking => "Parking",
            Self::GasStation => "GasStation",
            Self::ElectricVehicleChargingStation => "ElectricVehicleChargingStation",
            Self::CarDealer => "CarDealer",
            Self::CarRental => "CarRental",
            Self::DetailingStudio => "DetailingStudio",
            Self::RimsShop => "RimsShop",
            Self::Tuning => "Tuning",
            Self::TireShop => "TireShop",
            Self::CarInspectionStation => "CarInspectionStation",
        }
    }
}