use crate::{config::DatabaseConfig, errors::AppError, types::Database};
extern crate lazy_static;

use anyhow::Context;
use std::sync::Arc;
use surrealdb::opt::auth::Root;

pub async fn initialize_db() -> Result<Arc<Database>, AppError> {
    let config = DatabaseConfig::from_env().context("Failed to load database configuration")?;

    tracing::debug!("Connecting to SurrealDB: {}", config.endpoint);

    // let db = Surreal::new::<Ws>(&config.endpoint)
    //     .await
    //     .context("Failed to connect to SurrealDB")?;
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
    tracing::info!("Successfully connected to SurrealDB");

    let database = Database { connection: db };

    Ok(Arc::new(database))
}
