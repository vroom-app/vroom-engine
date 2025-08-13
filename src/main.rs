use std::sync::Arc;

use axum::{
    routing::{get, post, put},
    Router,
};
use sqlx::migrate::Migrator;
use tower::ServiceBuilder;
use tower_http::cors::CorsLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use vroomgine::{
    config::Config,
    database::create_pool,
    handlers,
    state::AppState,
};

static MIGRATOR: Migrator = sqlx::migrate!();

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "vroomgine=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Load configuration
    let config = Config::from_env()?;
    tracing::info!("Configuration loaded successfully.");

    // Create database connection pool
    let pool = create_pool(&config.database_url).await?;
    tracing::info!("Database connection pool created.");

    // Run migrations
    MIGRATOR.run(&pool).await?;
    tracing::info!("Database migration successfully done.");

    // Create shared application state
    let state = Arc::new(AppState::new(pool, config.clone()));

    // Build application routes
    let app = Router::new()
        .route("/search/health", get(handlers::health::health_check))
        .route("/search/businesses/sync", post(handlers::business::sync_businesses))
        .route("/search/businesses/sync", put(handlers::business::sync_user_business))
        .route("/search/businesses", get(handlers::business::list_businesses))
        .route("/search/businesses/search/radius", get(handlers::business::search_businesses_by_radius))
        .route("/search/businesses/search/radius-category", get(handlers::business::search_businesses_by_radius_and_category))
        .with_state(state)
        .layer(
            ServiceBuilder::new()
                .layer(CorsLayer::permissive())
        );

    // Start server
    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", config.server_port))
        .await?;
    
    tracing::info!("Server starting on port {}", config.server_port);
    
    axum::serve(listener, app).await?;

    Ok(())
}