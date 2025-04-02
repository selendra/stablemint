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
    pub max_connections: usize,
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
            max_connections: env::var("MAX_DB_POOL")
                .unwrap_or_else(|_| "5".to_string())
                .parse::<usize>()
                .unwrap_or(5),
        })
    }

    pub fn new_direct(
        endpoint: String,
        username: String,
        password: String,
        namespace: String,
        database: String,
        max_connections: usize,
    ) -> Self {
        Self {
            endpoint,
            username,
            password,
            namespace,
            database,
            max_connections,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Server {
    pub port: String,
    pub address: String,
}

impl Server {
    pub fn from_env() -> Result<Self> {
        // Load .env file only once per process
        dotenv().ok();

        Ok(Self {
            port: env::var("PORT").unwrap_or_else(|_| "3000".to_string()),
            address: env::var("ADDRESS").unwrap_or_else(|_| "0.0.0.0".to_string()),
        })
    }
}

pub struct SentryConfig {
    pub sentry_dsn: String,
}

impl SentryConfig {
    pub fn from_env() -> Result<Self> {
        dotenv().ok();

        Ok(Self {
            sentry_dsn: env::var("SENTRY_DSN").unwrap_or_else(|_| "".to_string()),
        })
    }
}
