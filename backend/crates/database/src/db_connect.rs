use app_config::DatabaseConfig;
use anyhow::Context;
use app_error::AppError;
use std::sync::Arc;

use crate::{Database, service::DbCredentials};

// Common setup code extracted to reduce duplication
async fn setup_db_config() -> Result<(DatabaseConfig, bool, usize), AppError> {
    let config = DatabaseConfig::from_env().context("Failed to load database configuration")?;

    // Validate configuration in production mode
    if cfg!(not(debug_assertions)) {
        config.validate()?;
    }

    tracing::debug!("Connecting to SurrealDB: {}", config.endpoint);

    // Check if using secure connection
    let is_secure = config.endpoint.starts_with("wss://");

    if is_secure {
        tracing::info!("Using secure TLS connection to database");
    } else if !config.endpoint.contains("memory") && cfg!(not(debug_assertions)) {
        tracing::warn!("Using non-secure database connection in production environment");
    }

    // Get pool size
    let max_connections = std::env::var("DB_POOL_SIZE")
        .map(|size| size.parse::<usize>().unwrap_or(10))
        .unwrap_or(10);

    tracing::info!(
        "Initializing database connection pool with {} connections",
        max_connections
    );

    Ok((config, is_secure, max_connections))
}

pub async fn initialize_db() -> Result<Arc<Database>, AppError> {
    let (config, _is_secure, max_connections) = setup_db_config().await?;
    
    // Load secure credentials
    let credentials = DbCredentials::from_env()?;

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

pub async fn initialize_memory_db() -> Result<Arc<Database>, AppError> {
    let (config, _is_secure, max_connections) = setup_db_config().await?;

    let db = Database::initialize_memmory_db(max_connections, &config.namespace, &config.database)
        .await?;

    tracing::info!("Successfully connected to in-memory SurrealDB with connection pool");

    Ok(Arc::new(db))
}