use app_config::AppConfig;
use anyhow::Context;
use app_error::AppError;
use std::sync::Arc;

use crate::{Database, service::DbCredentials};

// Common setup code extracted to reduce duplication
async fn setup_db_config() -> Result<(AppConfig, bool, usize), AppError> {
    // Load configuration from JSON file
    let config = AppConfig::load().context("Failed to load configuration")?;
    

    tracing::debug!("Connecting to SurrealDB: {}", config.database.endpoint);

    // Check if using secure connection
    let is_secure = config.database.endpoint.starts_with("wss://");

    if is_secure {
        tracing::info!("Using secure TLS connection to database");
    } else if !config.database.endpoint.contains("memory") && config.environment == "production" {
        tracing::warn!("Using non-secure database connection in production environment");
    }

    // Get pool size from configuration
    let max_connections = config.database.pool.size;

    tracing::info!(
        "Initializing database connection pool with {} connections",
        max_connections
    );

    Ok((config, is_secure, max_connections))
}

pub async fn initialize_db() -> Result<Arc<Database>, AppError> {
    let (config, _is_secure, max_connections) = setup_db_config().await?;
    
    // Create credentials from configuration
    let credentials = DbCredentials::new(
        config.database.username, 
        config.database.password
    );

    let db = Database::initialize(
        &config.database.endpoint,
        max_connections,
        &config.database.namespace,
        &config.database.database,
        &credentials,
    )
    .await?;

    tracing::info!("Successfully connected to SurrealDB with connection pool");

    Ok(Arc::new(db))
}

pub async fn initialize_memory_db() -> Result<Arc<Database>, AppError> {
    let db = Database::initialize_memmory_db(
        10, 
        "root", 
        "root"
    )
    .await?;

    tracing::info!("Successfully connected to in-memory SurrealDB with connection pool");

    Ok(Arc::new(db))
}