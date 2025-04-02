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

    tracing::info!(
        "Initializing database connection pool with {} connections",
        max_connections
    );

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
    let config = DatabaseConfig::from_env().context("Failed to load database configuration")?;

    // Validate configuration in production mode
    if cfg!(not(debug_assertions)) {
        config.validate()?;
    }

    tracing::debug!("Connecting to SurrealDB: {}", config.endpoint);

    // Load secure credentials
    let _credentials = DbCredentials::from_env()?;

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

    tracing::info!(
        "Initializing database connection pool with {} connections",
        max_connections
    );

    let db = Database::initialize_memmory_db(max_connections, &config.namespace, &config.database)
        .await?;

    tracing::info!("Successfully connected to SurrealDB with connection pool");

    Ok(Arc::new(db))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    use tokio::test;

    #[test]
    async fn test_initialize_db() -> Result<(), AppError> {
        // Set up environment variables for testing
        unsafe {
            env::set_var("DB_ENDPOINT", "memory");
        }
        unsafe { env::set_var("DB_NAMESPACE", "test_namespace") };
        unsafe { env::set_var("DB_NAME", "test_db") };
        unsafe { env::set_var("SURREALDB_USERNAME", "test_user") };
        unsafe { env::set_var("SURREALDB_PASSWORD", "test_password") };
        unsafe { env::set_var("DB_POOL_SIZE", "5") };

        // Call the function to initialize the database
        let db = initialize_memory_db().await?;

        // Assert that the database was initialized correctly
        assert!(
            Arc::strong_count(&db) >= 1,
            "Database should be wrapped in an Arc"
        );

        // Test a basic database operation to verify connectivity
        let conn = db.get_connection().await?;
        let version_info = conn.get_ref().version().await;
        assert!(version_info.is_ok(), "Database connection should be valid");

        // Clean up environment variables
        unsafe { env::remove_var("DB_ENDPOINT") };
        unsafe { env::remove_var("DB_NAMESPACE") };
        unsafe { env::remove_var("DB_NAME") };
        unsafe { env::remove_var("SURREALDB_USERNAME") };
        unsafe { env::remove_var("SURREALDB_PASSWORD") };
        unsafe { env::remove_var("DB_POOL_SIZE") };

        Ok(())
    }

    #[test]
    async fn test_initialize_db_invalid_config() {
        // Set up environment variables with invalid values
        unsafe { env::set_var("DB_ENDPOINT", "") }; // Empty endpoint
        unsafe { env::set_var("DB_NAMESPACE", "test_namespace") };
        unsafe { env::set_var("DB_NAME", "test_db") };
        unsafe { env::set_var("SURREALDB_USERNAME", "test_user") };
        unsafe { env::set_var("SURREALDB_PASSWORD", "test_password") };

        // Call the function to initialize the database
        let result = initialize_db().await;

        // Assert that an error was returned
        assert!(result.is_err(), "Should fail with invalid configuration");

        // Clean up environment variables
        unsafe { env::remove_var("DB_ENDPOINT") };
        unsafe { env::remove_var("DB_NAMESPACE") };
        unsafe { env::remove_var("DB_NAME") };
        unsafe { env::remove_var("SURREALDB_USERNAME") };
        unsafe { env::remove_var("SURREALDB_PASSWORD") };
    }

    #[test]
    async fn test_initialize_db_with_mock() -> Result<(), AppError> {
        unsafe {
            env::set_var("DB_ENDPOINT", "memory");
        }
        unsafe { env::set_var("DB_NAMESPACE", "test_namespace") };
        unsafe { env::set_var("DB_NAME", "test_db") };
        unsafe { env::set_var("SURREALDB_USERNAME", "test_user") };
        unsafe { env::set_var("SURREALDB_PASSWORD", "test_password") };
        unsafe { env::set_var("DB_POOL_SIZE", "3") }; // Small pool for testing

        // Initialize the database
        let db = initialize_memory_db().await?;

        // Test the pool by getting multiple connections
        let conn1 = db.get_connection().await?;
        let conn2 = db.get_connection().await?;
        let conn3 = db.get_connection().await?;

        // This should still work but will create a new connection since pool is size 3
        let conn4 = db.get_connection().await?;

        // Return connections to the pool
        drop(conn1); // This will be returned to the pool
        drop(conn2); // This will be returned to the pool

        // Get another connection, which should come from the pool
        let _conn5 = db.get_connection().await?;

        // Clean up
        drop(conn3);
        drop(conn4);
        drop(_conn5);
        drop(db); // This should drop the remaining connections

        unsafe { env::remove_var("DB_ENDPOINT") };
        unsafe { env::remove_var("DB_NAMESPACE") };
        unsafe { env::remove_var("DB_NAME") };
        unsafe { env::remove_var("SURREALDB_USERNAME") };
        unsafe { env::remove_var("SURREALDB_PASSWORD") };
        unsafe { env::remove_var("DB_POOL_SIZE") };

        Ok(())
    }
}
