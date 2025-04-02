use app_config::DatabaseConfig;

use anyhow::Context;
use app_error::AppError;
use std::sync::Arc;

use crate::{Database, service::DbCredentials};

pub async fn initialize_db() -> Result<Arc<Database>, AppError> {
    let config = DatabaseConfig::from_env().context("Failed to load database configuration")?;
    
    // Validate configuration in production mode
    if cfg!(not(debug_assertions)) {
        config.validate()?;
    }

    tracing::debug!("Connecting to SurrealDB: {}", config.endpoint);

    // Load secure credentials
    let credentials = DbCredentials::from_env()?;
    
    // Add TLS verification options if using secure connection
    let is_secure = config.endpoint.starts_with("wss://");
    
    if is_secure {
        tracing::info!("Using secure TLS connection to database");
    } else if !config.endpoint.contains("memory") && cfg!(not(debug_assertions)) {
        tracing::warn!("Using non-secure database connection in production environment");
    }

    // Use the new connection pooling mechanism with configurable pool size
    let max_connections = std::env::var("DB_POOL_SIZE")
        .map(|size| size.parse::<usize>().unwrap_or(10))
        .unwrap_or(10);
        
    tracing::info!("Initializing database connection pool with {} connections", max_connections);
    
    let db = Database::initialize(
        &config.endpoint,
        max_connections,
        &config.namespace,
        &config.database,
        &credentials,
    )
    .await?;

    tracing::info!("Successfully connected to SurrealDB with connection pool");

    Ok(Arc::new(db))
}