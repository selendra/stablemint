use anyhow::Context;
use app_config::{AppConfig,  SurrealDbConfig};
use app_error::AppError;
use std::sync::Arc;

use crate::{service::DbCredentials, Database};

// Common setup code extracted to reduce duplication
async fn setup_db_config(db_config: &SurrealDbConfig) -> Result<(bool, usize), AppError> {
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

pub async fn initialize_user_db() -> Result<Arc<Database>, AppError> {
    // Load configuration from JSON file
    let config = AppConfig::load().context("Failed to load configuration")?;
    
    let db_config = config.database.user_db;
    let (_is_secure, max_connections) = setup_db_config(&db_config).await?;

    // Create credentials from configuration
    let credentials = DbCredentials::new(db_config.username, db_config.password);

    let db = Database::initialize(
        &db_config.endpoint,
        max_connections,
        &db_config.namespace,
        &db_config.database,
        &credentials,
    )
    .await?;

    tracing::info!("Successfully connected to User SurrealDB with connection pool");

    Ok(Arc::new(db))
}

pub async fn initialize_wallet_db() -> Result<Arc<Database>, AppError> {
    // Load configuration from JSON file
    let config = AppConfig::load().context("Failed to load configuration")?;
    
    let db_config = config.database.wallet_db;
    let (_is_secure, max_connections) = setup_db_config(&db_config).await?;

    // Create credentials from configuration
    let credentials = DbCredentials::new(
        db_config.username.clone(), 
        db_config.password.clone()
    );

    let db = Database::initialize(
        &db_config.endpoint,
        max_connections,
        &db_config.namespace,
        &db_config.database,
        &credentials,
    )
    .await?;

    tracing::info!("Successfully connected to Wallet SurrealDB with connection pool");

    Ok(Arc::new(db))
}

pub async fn initialize_memory_db() -> Result<Arc<Database>, AppError> {
    let db = Database::initialize_memmory_db(10, "root", "root").await?;

    tracing::info!("Successfully connected to in-memory SurrealDB with connection pool");

    Ok(Arc::new(db))
}
