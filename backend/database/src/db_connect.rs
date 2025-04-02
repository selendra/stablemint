use app_config::DatabaseConfig;

use anyhow::Context;
use app_error::AppError;
use std::sync::Arc;
use surrealdb::opt::auth::Root;

use crate::Database;

pub async fn initialize_db() -> Result<Arc<Database>, AppError> {
    let config = DatabaseConfig::from_env().context("Failed to load database configuration")?;

    tracing::debug!("Connecting to SurrealDB: {}", config.endpoint);
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

pub async fn initialize_memmory_db() -> Result<Arc<Database>, AppError> {
    let endpoint = "memory";
    let namespace = "memory-namespace";
    let database = "memory-database";

    tracing::debug!("Connecting to SurrealDB: {}", endpoint);
    let db = surrealdb::engine::any::connect(endpoint)
        .await
        .context("Failed to connect to SurrealDB")?;

    // Use a single operation to select namespace and database
    db.use_ns(namespace)
        .use_db(database)
        .await
        .context("Failed to select namespace and database")?;
    tracing::info!("Successfully connected to SurrealDB");

    let database = Database { connection: db };

    Ok(Arc::new(database))
}


#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::Result;

    #[tokio::test]
    async fn test_initialize_memory_db() -> Result<()> {
        // Call the function to initialize the memory database
        let db_result = initialize_memmory_db().await;
        
        // Check if the connection was successful
        assert!(db_result.is_ok(), "Failed to connect to memory database");
        
        // Get the database from the result
        let db = db_result.unwrap();
        
        // Optional: Perform a simple query to verify the database is working
        let query_result = db.connection.query("INFO FOR DB;").await;
        assert!(query_result.is_ok(), "Failed to execute query on memory database");
        
        Ok(())
    }
}