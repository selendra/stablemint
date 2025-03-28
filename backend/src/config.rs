use anyhow::Result;
use dotenv::dotenv;
use serde::{Deserialize, Serialize};
use std::env;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    pub endpoint: String,
    pub username: String,
    pub password: String,
    pub namespace: String,
    pub database: String,
}

impl DatabaseConfig {
    pub fn from_env() -> Result<Self> {
        // Load .env file only once per process
        dotenv().ok();

        Ok(Self {
            endpoint: env::var("SURREALDB_ENDPOINT")
                .unwrap_or_else(|_| "ws://localhost:8000".to_string()),
            username: env::var("SURREALDB_USERNAME").unwrap_or_else(|_| "root".to_string()),
            password: env::var("SURREALDB_PASSWORD").unwrap_or_else(|_| "root".to_string()),
            namespace: env::var("SURREALDB_NAMESPACE").unwrap_or_else(|_| "selendraDb".to_string()),
            database: env::var("SURREALDB_DATABASE").unwrap_or_else(|_| "cryptoBank".to_string()),
        })
    }
}
