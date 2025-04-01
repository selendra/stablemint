// database/src/conn.rs

use crate::credentials::SecureDatabaseConfig;
use crate::types::Database;
use anyhow::Result;
use stablemint_error::AppError;
use std::sync::Arc;
use surrealdb::{Surreal, engine::any::Any, opt::auth::Root};

/// Original DatabaseConfig for backward compatibility
#[derive(Clone)]
pub struct DatabaseConfig {
    pub endpoint: String,
    pub username: String,
    pub password: String,
    pub namespace: String,
    pub database: String,
}

impl DatabaseConfig {
    pub fn new(
        endpoint: impl Into<String>,
        username: impl Into<String>,
        password: impl Into<String>,
        namespace: impl Into<String>,
        database: impl Into<String>,
    ) -> Self {
        Self {
            endpoint: endpoint.into(),
            username: username.into(),
            password: password.into(),
            namespace: namespace.into(),
            database: database.into(),
        }
    }

    /// Convert to the new secure config for enhanced security features
    pub fn to_secure_config(self) -> SecureDatabaseConfig {
        use crate::credentials::DatabaseCredentials;

        let credentials = DatabaseCredentials::new_direct(self.username, self.password);
        SecureDatabaseConfig::new(self.endpoint, credentials, self.namespace, self.database)
    }
}

/// Initialize database with original config (for backward compatibility)
pub async fn initialize_db(config: DatabaseConfig) -> Result<Arc<Database>, AppError> {
    Ok(connect_and_setup(&config)
        .await
        .map(|connection| Arc::new(Database { connection }))?)
}

/// Initialize database with secure config (preferred method)
pub async fn initialize_secure_db(config: SecureDatabaseConfig) -> Result<Arc<Database>, AppError> {
    let db_config = config.to_database_config();
    initialize_db(db_config).await
}

async fn connect_and_setup(config: &DatabaseConfig) -> Result<Surreal<Any>> {
    tracing::info!(
        endpoint = %config.endpoint,
        namespace = %config.namespace,
        database = %config.database,
        "Connecting to SurrealDB"
    );

    // Connect to the database
    let db = surrealdb::engine::any::connect(&config.endpoint)
        .await
        .map_err(|e| {
            tracing::error!(
                error = %e,
                endpoint = %config.endpoint,
                "Failed to connect to SurrealDB"
            );
            anyhow::anyhow!("Failed to connect to SurrealDB: {}", e)
        })?;

    if config.endpoint != "memory" {
        // Authenticate to the database
        tracing::debug!(
            endpoint = %config.endpoint,
            username = %config.username,
            "Authenticating to SurrealDB"
        );

        db.signin(Root {
            username: &config.username,
            password: &config.password,
        })
        .await
        .map_err(|e| {
            tracing::error!(
                error = %e,
                endpoint = %config.endpoint,
                username = %config.username,
                "Authentication failed"
            );
            anyhow::anyhow!("Failed to authenticate to SurrealDB: {}", e)
        })?;
    }

    // Use a single operation to select namespace and database
    tracing::debug!(
        namespace = %config.namespace,
        database = %config.database,
        "Selecting namespace and database"
    );

    db.use_ns(&config.namespace)
        .use_db(&config.database)
        .await
        .map_err(|e| {
            tracing::error!(
                error = %e,
                namespace = %config.namespace,
                database = %config.database,
                "Failed to select namespace and database"
            );
            anyhow::anyhow!("Failed to select namespace and database: {}", e)
        })?;

    tracing::info!(
        endpoint = %config.endpoint,
        namespace = %config.namespace,
        database = %config.database,
        "Successfully connected to SurrealDB"
    );

    Ok(db)
}
