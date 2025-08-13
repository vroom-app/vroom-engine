use std::env;
use crate::shared::error::{AppError, Result};

#[derive(Debug, Clone)]
pub struct Config {
    pub database_url: String,
    pub server_port: u16,
    pub overpass_api_url: String,
    pub overpass_timeout: u64,
}

impl Config {
    pub fn from_env() -> Result<Self> {
        dotenvy::dotenv().ok();

        let database_url = env::var("DATABASE_URL")
            .map_err(|_| AppError::Config("DATABASE_URL must be set".to_string()))?;

        let server_port = env::var("SERVER_PORT")
            .unwrap_or_else(|_| "3000".to_string())
            .parse::<u16>()
            .map_err(|_| AppError::Config("Invalid SERVER_PORT".to_string()))?;

        Ok(Config {
            database_url,
            server_port,
            overpass_api_url: "https://overpass-api.de/api/interpreter".to_string(),
            overpass_timeout: 50,
        })
    }
}