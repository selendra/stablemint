use app_config::DatabaseConfig;

use anyhow::Context;
use app_error::AppError;
use std::sync::Arc;

use crate::Database;

pub async fn initialize_db() -> Result<Arc<Database>, AppError> {
    let config = DatabaseConfig::from_env().context("Failed to load database configuration")?;

    tracing::debug!("Connecting to SurrealDB: {}", config.endpoint);

    // Use the new connection pooling mechanism
    let max_connections = 10; // You may want to make this configurable
    let db = Database::initialize(
        &config.endpoint,
        max_connections,
        &config.namespace,
        &config.database,
        &config.username,
        &config.password,
    )
    .await?;

    tracing::info!("Successfully connected to SurrealDB with connection pool");

    Ok(Arc::new(db))
}
