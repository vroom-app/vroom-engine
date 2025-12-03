use axum::{
    Json, extract::rejection::JsonRejection, http::StatusCode, response::{IntoResponse, Response}
};
use serde::Serialize;
use serde_json::json;
use thiserror::Error;

pub type Result<T> = std::result::Result<T, AppError>;

#[derive(Debug, Serialize)]
pub struct ErrorDto {
    pub code: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<serde_json::Value>,
}

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
        let (status, code, message) = match &self {
            AppError::Database(_) => (StatusCode::INTERNAL_SERVER_ERROR, "DATABASE_ERROR", "Database error occurred"),
            AppError::OsmParsing(_) => (StatusCode::BAD_REQUEST, "OSM_PARSING_FAILED", "Failed to parse OSM data"),
            AppError::Http(_) => (StatusCode::BAD_GATEWAY, "EXTERNAL_SERVICE_ERROR", "External service error"),
            AppError::Io(_) => (StatusCode::INTERNAL_SERVER_ERROR, "IO_ERROR", "File system error"),
            AppError::Config(_) => (StatusCode::INTERNAL_SERVER_ERROR, "CONFIG_ERROR", "Configuration error"),
            AppError::Import(_) => (StatusCode::INTERNAL_SERVER_ERROR, "IMPORT_FAILED", "Import failed"),
        };

        let body = Json(ErrorDto {
            code: code.to_string(),
            message: message.to_string(),
            details: Some(json!(self.to_string())),
        });

        (status, body).into_response()
    }
}

impl From<JsonRejection> for AppError {
    fn from(err: JsonRejection) -> Self {
        AppError::Import(format!("INVALID_JSON: {}", err))
    }
}