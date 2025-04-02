use anyhow::Result;
use dotenv::dotenv;
use serde::{Deserialize, Serialize};
use std::env;

use app_error::{AppError, AppResult};

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

    pub fn new_direct(
        endpoint: String,
        username: String,
        password: String,
        namespace: String,
        database: String,
    ) -> Self {
        Self {
            endpoint,
            username,
            password,
            namespace,
            database,
        }
    }

    // Validate configuration for production use
    pub fn validate(&self) -> AppResult<()> {
        let mut errors = Vec::new();

        // Validate endpoint
        if self.endpoint.trim().is_empty() {
            errors.push("Database endpoint cannot be empty".to_string());
        } else if !self.endpoint.starts_with("wss://") && !self.endpoint.contains("memory") {
            errors.push("Production should use a secure 'wss://' connection".to_string());
        }

        // Validate namespace
        if self.namespace.trim().is_empty() {
            errors.push("Database namespace cannot be empty".to_string());
        }

        // Validate database
        if self.database.trim().is_empty() {
            errors.push("Database name cannot be empty".to_string());
        }

        // Validate credentials
        if self.username == "root" {
            errors.push("Using default 'root' username in production is insecure".to_string());
        }

        if self.password == "root" {
            errors.push("Using default 'root' password in production is insecure".to_string());
        }

        if !errors.is_empty() {
            return Err(AppError::ConfigError(anyhow::anyhow!(
                "Invalid database configuration: {}",
                errors.join(", ")
            )));
        }

        Ok(())
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

    // Validate server configuration
    pub fn validate(&self) -> AppResult<()> {
        // Validate port
        match self.port.parse::<u16>() {
            Ok(_) => {} // Valid port number
            Err(_) => {
                return Err(AppError::ConfigError(anyhow::anyhow!(
                    "Invalid server port: '{}' - must be a valid port number",
                    self.port
                )));
            }
        }

        // Validate address (basic check)
        if self.address.trim().is_empty() {
            return Err(AppError::ConfigError(anyhow::anyhow!(
                "Server address cannot be empty"
            )));
        }

        Ok(())
    }
}

pub struct SentryConfig {
    pub sentry_dsn: String,
}

impl SentryConfig {
    pub fn from_env() -> Result<Self> {
        dotenv().ok();

        Ok(Self {
            sentry_dsn: env::var("SENTRY_DSN")?,
        })
    }

    pub fn validate(&self) -> AppResult<()> {
        if cfg!(not(debug_assertions)) && self.sentry_dsn.trim().is_empty() {
            return Err(AppError::ConfigError(anyhow::anyhow!(
                "Sentry DSN should be configured in production for error monitoring"
            )));
        }

        Ok(())
    }
}
