use std::env;
use crate::error::{AppError, Result};

#[derive(Debug, Clone)]
pub struct Config {
    pub database_url: String,
    pub osm_data_url: String,
    pub server_port: u16
}

impl Config {
    pub fn from_env() -> Result<Self> {
        dotenvy::dotenv().ok();

        let database_url = env::var("DATABASE_URL")
            .map_err(|_| AppError::Config("DATABASE_URL must be set".to_string()))?;

        let osm_data_url = env::var("OSM_DATA_URL")
            .unwrap_or_else(|_| "https://download.geofabrik.de/europe/bulgaria-latest.osm.pbf".to_string());

        let server_port = env::var("SERVER_PORT")
            .unwrap_or_else(|_| "3000".to_string())
            .parse::<u16>()
            .map_err(|_| AppError::Config("Invalid SERVER_PORT".to_string()))?;

        Ok(Config {
            database_url,
            osm_data_url,
            server_port,
        })
    }
}