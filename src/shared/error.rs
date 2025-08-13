use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),
    
    #[error("OSM parsing error: {0}")]
    OsmParsing(String),
    
    #[error("HTTP request error: {0}")]
    Http(#[from] reqwest::Error),
    
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("Configuration error: {0}")]
    Config(String),
    
    #[error("Import error: {0}")]
    Import(String),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            AppError::Database(ref err) => {
                tracing::error!("Database error: {}", err);
                (StatusCode::INTERNAL_SERVER_ERROR, "Database error occurred")
            }
            AppError::OsmParsing(ref err) => {
                tracing::error!("OSM parsing error: {}", err);
                (StatusCode::BAD_REQUEST, "Failed to parse OSM data")
            }
            AppError::Http(ref err) => {
                tracing::error!("HTTP error: {}", err);
                (StatusCode::BAD_GATEWAY, "External service error")
            }
            AppError::Io(ref err) => {
                tracing::error!("IO error: {}", err);
                (StatusCode::INTERNAL_SERVER_ERROR, "File system error")
            }
            AppError::Config(ref err) => {
                tracing::error!("Configuration error: {}", err);
                (StatusCode::INTERNAL_SERVER_ERROR, "Configuration error")
            }
            AppError::Import(ref err) => {
                tracing::error!("Import error: {}", err);
                (StatusCode::INTERNAL_SERVER_ERROR, "Import failed")
            }
        };

        let body = Json(json!({
            "error": error_message,
            "details": self.to_string()
        }));

        (status, body).into_response()
    }
}

pub type Result<T> = std::result::Result<T, AppError>;