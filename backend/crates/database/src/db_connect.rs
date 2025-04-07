use anyhow::Context;
use app_config::AppConfig;
use app_config::config_loader::DatabaseConfig;
use app_error::AppError;
use std::sync::Arc;

use crate::{Database, service::DbCredentials};

// Common setup code extracted to reduce duplication
async fn setup_db_config(db_config: &DatabaseConfig) -> Result<(bool, usize), AppError> {
    tracing::debug!("Connecting to SurrealDB: {}", db_config.endpoint);

    // Check if using secure connection
    let is_secure = db_config.endpoint.starts_with("wss://");

    if is_secure {
        tracing::info!("Using secure TLS connection to database");
    } else if !db_config.endpoint.contains("memory") {
        tracing::warn!("Using non-secure database connection");
    }

    // Get pool size from configuration
    let max_connections = db_config.pool.size;

    tracing::info!(
        "Initializing database connection pool with {} connections",
        max_connections
    );

    Ok((is_secure, max_connections))
}

// Initialize database with specific config for a microservice
pub async fn initialize_db_with_config(db_config: &DatabaseConfig) -> Result<Arc<Database>, AppError> {
    let (_is_secure, max_connections) = setup_db_config(db_config).await?;

    // Create credentials from configuration
    let credentials = DbCredentials::new(&db_config.username, &db_config.password);

    let db = Database::initialize(
        &db_config.endpoint,
        max_connections,
        &db_config.namespace,
        &db_config.database,
        &credentials,
    )
    .await?;

    tracing::info!(
        "Successfully connected to SurrealDB at {} with connection pool",
        db_config.endpoint
    );

    Ok(Arc::new(db))
}

// Initialize database for a specific microservice using the service name
pub async fn initialize_service_db(config: &AppConfig, service_name: &str) -> Result<Arc<Database>, AppError> {
    let db_config = config.get_database_config(service_name);
    
    tracing::info!(
        "Initializing database for '{}' service at {}",
        service_name,
        db_config.endpoint
    );
    
    initialize_db_with_config(&db_config).await
}

// Legacy function for backward compatibility
pub async fn initialize_db() -> Result<Arc<Database>, AppError> {
    // Load configuration from JSON file
    let config = AppConfig::load().context("Failed to load configuration")?;
    
    // Use the main database config
    initialize_db_with_config(&config.database).await
}

pub async fn initialize_memory_db() -> Result<Arc<Database>, AppError> {
    let db = Database::initialize_memmory_db(10, "root", "root").await?;

    tracing::info!("Successfully connected to in-memory SurrealDB with connection pool");

    Ok(Arc::new(db))
}
