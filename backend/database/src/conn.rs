use std::sync::Arc;
use anyhow::{Context, Result};
use stablemint_error::AppError;
use surrealdb::{
    engine::any::Any,
    opt::auth::Root,
    Surreal,
};
use crate::types::Database;

/// Configuration for SurrealDB connection
#[derive(Clone, Debug)]
pub struct DatabaseConfig {
    pub endpoint: String,
    pub username: String,
    pub password: String,
    pub namespace: String,
    pub database: String,
}

impl DatabaseConfig {
    /// Create a new database configuration with the provided parameters
    ///
    /// # Arguments
    ///
    /// * `endpoint` - SurrealDB endpoint URL
    /// * `username` - Username for authentication
    /// * `password` - Password for authentication
    /// * `namespace` - SurrealDB namespace
    /// * `database` - SurrealDB database name
    ///
    /// # Returns
    ///
    /// * `Result<Self>` - The configured DatabaseConfig or an error
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
    
    /// Create a DatabaseConfig from environment variables
    ///
    /// # Environment Variables
    ///
    /// * `SURREALDB_ENDPOINT` - SurrealDB endpoint URL
    /// * `SURREALDB_USERNAME` - Username for authentication
    /// * `SURREALDB_PASSWORD` - Password for authentication
    /// * `SURREALDB_NAMESPACE` - SurrealDB namespace
    /// * `SURREALDB_DATABASE` - SurrealDB database name
    ///
    /// # Returns
    ///
    /// * `Result<Self>` - The configured DatabaseConfig or an error
    pub fn from_env() -> Result<Self> {
        use std::env;
        
        Ok(Self {
            endpoint: env::var("SURREALDB_ENDPOINT").context("SURREALDB_ENDPOINT must be set")?,
            username: env::var("SURREALDB_USERNAME").context("SURREALDB_USERNAME must be set")?,
            password: env::var("SURREALDB_PASSWORD").context("SURREALDB_PASSWORD must be set")?,
            namespace: env::var("SURREALDB_NAMESPACE").context("SURREALDB_NAMESPACE must be set")?,
            database: env::var("SURREALDB_DATABASE").context("SURREALDB_DATABASE must be set")?,
        })
    }
}


pub async fn initialize_db(config: DatabaseConfig) -> Result<Arc<Database>, AppError> {
    Ok(connect_and_setup(&config).await.map(|connection| {
        Arc::new(Database { connection })
    })?)
}


async fn connect_and_setup(config: &DatabaseConfig) -> Result<Surreal<Any>> {
    tracing::debug!("Connecting to SurrealDB: {}", config.endpoint);
    
    // Connect to the database
    let db = surrealdb::engine::any::connect(&config.endpoint)
        .await
        .context("Failed to connect to SurrealDB")?;

    // Authenticate to the database
    db.signin(Root {
        username: &config.username,
        password: &config.password,
    })
    .await
    .context("Failed to authenticate to SurrealDB")?;

    // Use a single operation to select namespace and database
    db.use_ns(&config.namespace)
        .use_db(&config.database)
        .await
        .context("Failed to select namespace and database")?;
    
    tracing::info!(
        "Successfully connected to SurrealDB (ns: {}, db: {})",
        config.namespace,
        config.database
    );
    
    Ok(db)
}


pub async fn test_connection(config: &DatabaseConfig) -> Result<(), AppError> {
    connect_and_setup(config)
        .await
        .map(|_| ())
        .map_err(|e| AppError::Database(e))
}
