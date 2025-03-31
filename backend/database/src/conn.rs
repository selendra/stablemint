use crate::types::Database;
use anyhow::{Context, Result};
use dotenv::dotenv;
use stablemint_error::AppError;
use std::sync::Arc;
use surrealdb::{Surreal, engine::any::Any, opt::auth::Root};

/// Configuration for SurrealDB connection
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

    pub fn from_env() -> Result<Self> {
        use std::env;
        dotenv().ok();

        Ok(Self {
            endpoint: env::var("SURREALDB_ENDPOINT").context("SURREALDB_ENDPOINT must be set")?,
            username: env::var("SURREALDB_USERNAME").context("SURREALDB_USERNAME must be set")?,
            password: env::var("SURREALDB_PASSWORD").context("SURREALDB_PASSWORD must be set")?,
            namespace: env::var("SURREALDB_NAMESPACE")
                .context("SURREALDB_NAMESPACE must be set")?,
            database: env::var("SURREALDB_DATABASE").context("SURREALDB_DATABASE must be set")?,
        })
    }
}

pub async fn initialize_db(config: DatabaseConfig) -> Result<Arc<Database>, AppError> {
    Ok(connect_and_setup(&config)
        .await
        .map(|connection| Arc::new(Database { connection }))?)
}

async fn connect_and_setup(config: &DatabaseConfig) -> Result<Surreal<Any>> {
    tracing::debug!("Connecting to SurrealDB: {}", config.endpoint);

    // Connect to the database
    let db = surrealdb::engine::any::connect(&config.endpoint)
        .await
        .context("Failed to connect to SurrealDB")?;

    if config.endpoint != "memory" {
        // Authenticate to the database
        db.signin(Root {
            username: &config.username,
            password: &config.password,
        })
        .await
        .context("Failed to authenticate to SurrealDB")?;
    }

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
