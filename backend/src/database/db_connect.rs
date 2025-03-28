use anyhow::{Context, Result};
use std::sync::Arc;
use surrealdb::opt::auth::Root;

use crate::{config::DatabaseConfig, types::Database};

pub async fn create_db_pool() -> Result<Database> {
    let config = DatabaseConfig::from_env().context("Failed to load database configuration")?;

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

    tracing::info!("Successfully connected to SurrealDB at {}", config.endpoint);

    Ok(Arc::new(db))
}
