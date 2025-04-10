use app_error::{AppError, AppResult};
use serde::{Deserialize, Serialize};

mod config_loader;
pub use config_loader::*;

/// The simplified configuration system uses only JSON configuration files
/// and doesn't rely on environment variables.
///
/// This module provides the core configuration types and loading functions.

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    pub endpoint: String,
    pub username: String,
    pub password: String,
    pub namespace: String,
    pub database: String,
}

impl DatabaseConfig {
    pub fn new(
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
    pub port: u16,
    pub address: String,
}

impl Server {
    pub fn new(address: String, port: u16) -> Self {
        Self { port, address }
    }

    // Validate server configuration
    pub fn validate(&self) -> AppResult<()> {
        // Validate port
        if self.port == 0 {
            return Err(AppError::ConfigError(anyhow::anyhow!(
                "Invalid server port: '0' is not a valid port number"
            )));
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

#[derive(Debug, Clone)]
pub struct JwtConfig {
    pub secret: Vec<u8>,
    pub expiry_hours: u64,
}

impl JwtConfig {
    pub fn new(secret: &[u8], expiry_hours: u64) -> Self {
        Self {
            secret: secret.to_vec(),
            expiry_hours,
        }
    }
}

/// Helper function for backward compatibility
/// Converts from the new AppConfig to the legacy DatabaseConfig
impl From<&AppConfig> for DatabaseConfig {
    fn from(config: &AppConfig) -> Self {
        Self {
            endpoint: config.database.user_db.endpoint.clone(),
            username: config.database.user_db.username.clone(),
            password: config.database.user_db.password.clone(),
            namespace: config.database.user_db.namespace.clone(),
            database: config.database.user_db.database.clone(),
        }
    }
}

/// Helper function for backward compatibility
/// Converts from the new AppConfig to the legacy Server config
impl From<&AppConfig> for Server {
    fn from(config: &AppConfig) -> Self {
        Self {
            port: config.server.port,
            address: config.server.host.clone(),
        }
    }
}

/// Helper function for backward compatibility
/// Converts from the new AppConfig to the legacy JwtConfig
impl From<&AppConfig> for JwtConfig {
    fn from(config: &AppConfig) -> Self {
        Self {
            secret: config.security.jwt.secret.clone().into_bytes(),
            expiry_hours: config.security.jwt.expiry_hours,
        }
    }
}
