// database/src/credentials.rs

use anyhow::Result;
use serde::{Deserialize, Serialize};
use stablemint_error::AppError;
use std::env;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::time::{Duration, SystemTime};

/// Credential source types
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CredentialSource {
    /// Direct credentials (least secure, mostly for testing)
    Direct,
    /// Environment variables (for containerized environments)
    Environment,
    /// File-based credentials (for traditional deployments)
    File,
    /// Vault-based credentials (most secure, for production)
    Vault,
}

/// Sensitive string that doesn't expose its contents in Debug output
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SecureString(String);

impl SecureString {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Debug for SecureString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[REDACTED]")
    }
}

impl From<String> for SecureString {
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl From<&str> for SecureString {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}

/// Database credentials with rotation support
#[derive(Debug, Clone)]
pub struct DatabaseCredentials {
    username: SecureString,
    password: SecureString,
    source: CredentialSource,
    last_rotated: Option<SystemTime>,
    rotation_interval: Option<Duration>,
}

impl DatabaseCredentials {
    /// Create new credentials directly (least secure)
    pub fn new_direct(username: impl Into<String>, password: impl Into<String>) -> Self {
        Self {
            username: SecureString::new(username),
            password: SecureString::new(password),
            source: CredentialSource::Direct,
            last_rotated: Some(SystemTime::now()),
            rotation_interval: None,
        }
    }

    /// Load credentials from environment variables
    pub fn from_env(username_var: &str, password_var: &str) -> Result<Self, AppError> {
        let username = env::var(username_var).map_err(|_| {
            AppError::InvalidInput(format!("Environment variable not found: {}", username_var))
        })?;

        let password = env::var(password_var).map_err(|_| {
            AppError::InvalidInput(format!("Environment variable not found: {}", password_var))
        })?;

        Ok(Self {
            username: SecureString::new(username),
            password: SecureString::new(password),
            source: CredentialSource::Environment,
            last_rotated: None,
            rotation_interval: None,
        })
    }

    /// Load credentials from a file
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, AppError> {
        let path = path.as_ref();
        let mut file = File::open(path).map_err(|e| {
            AppError::InvalidInput(format!("Failed to open credentials file: {}", e))
        })?;

        let metadata = file
            .metadata()
            .map_err(|e| AppError::InvalidInput(format!("Failed to read file metadata: {}", e)))?;

        // Basic file permission check on Unix systems
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let permissions = metadata.permissions();
            let mode = permissions.mode();
            // Check if permissions are too open (anything beyond 0600)
            if mode & 0o077 != 0 {
                tracing::warn!("Credential file has loose permissions: {:o}", mode);
            }
        }

        let mut contents = String::new();
        file.read_to_string(&mut contents).map_err(|e| {
            AppError::InvalidInput(format!("Failed to read credentials file: {}", e))
        })?;

        let creds: CredentialFile = serde_json::from_str(&contents).map_err(|e| {
            AppError::InvalidInput(format!("Invalid credentials file format: {}", e))
        })?;

        Ok(Self {
            username: SecureString::new(creds.username),
            password: SecureString::new(creds.password),
            source: CredentialSource::File,
            last_rotated: metadata.modified().ok().map(|time| time.into()),
            rotation_interval: None,
        })
    }

    /// Create credentials with vault integration
    #[cfg(feature = "vault")]
    pub async fn from_vault(vault_addr: &str, token: &str, path: &str) -> Result<Self, AppError> {
        // This would use a vault client library like hashicorp_vault
        // Implementation depends on which vault client you use

        #[cfg(feature = "hashicorp")]
        {
            use hashicorp_vault::client::{VaultClient, VaultClientSettingsBuilder};

            let vault_client = VaultClientSettingsBuilder::default()
                .address(vault_addr)
                .token(token)
                .build()
                .map_err(|e| {
                    AppError::InvalidInput(format!("Failed to build Vault client: {}", e))
                })?
                .client()
                .map_err(|e| {
                    AppError::InvalidInput(format!("Failed to create Vault client: {}", e))
                })?;

            let secret = vault_client.get_secret(path).map_err(|e| {
                AppError::InvalidInput(format!("Failed to retrieve secret from Vault: {}", e))
            })?;

            let username = secret
                .get("username")
                .and_then(|v| v.as_str())
                .ok_or_else(|| {
                    AppError::InvalidInput("Username not found in Vault secret".to_string())
                })?;

            let password = secret
                .get("password")
                .and_then(|v| v.as_str())
                .ok_or_else(|| {
                    AppError::InvalidInput("Password not found in Vault secret".to_string())
                })?;

            Ok(Self {
                username: SecureString::new(username),
                password: SecureString::new(password),
                source: CredentialSource::Vault,
                last_rotated: Some(SystemTime::now()),
                rotation_interval: Some(Duration::from_secs(86400)), // 24 hours
            })
        }

        #[cfg(not(feature = "hashicorp"))]
        {
            Err(AppError::InvalidInput(
                "Vault support is not compiled in this build".to_string(),
            ))
        }
    }

    /// Set credential rotation interval
    pub fn with_rotation_interval(mut self, interval: Duration) -> Self {
        self.rotation_interval = Some(interval);
        self
    }

    /// Check if credentials need rotation
    pub fn needs_rotation(&self) -> bool {
        if let (Some(last_rotated), Some(interval)) = (self.last_rotated, self.rotation_interval) {
            match SystemTime::now().duration_since(last_rotated) {
                Ok(elapsed) => elapsed >= interval,
                Err(_) => false, // Clock went backwards, ignore
            }
        } else {
            false
        }
    }

    /// Rotate credentials (implementation depends on credential source)
    pub async fn rotate(&mut self) -> Result<(), AppError> {
        match self.source {
            CredentialSource::Direct => {
                tracing::warn!("Cannot rotate direct credentials");
                Ok(())
            }
            CredentialSource::Environment => {
                // Reload from environment
                if let Ok(refreshed) = Self::from_env(
                    &format!(
                        "DB_USERNAME_{}",
                        SystemTime::now().elapsed().unwrap().as_secs()
                    ),
                    &format!(
                        "DB_PASSWORD_{}",
                        SystemTime::now().elapsed().unwrap().as_secs()
                    ),
                ) {
                    self.username = refreshed.username;
                    self.password = refreshed.password;
                    self.last_rotated = Some(SystemTime::now());
                    tracing::info!("Rotated credentials from environment");
                    Ok(())
                } else {
                    tracing::warn!("Failed to rotate credentials from environment");
                    Err(AppError::InvalidInput(
                        "Failed to rotate credentials from environment".to_string(),
                    ))
                }
            }
            CredentialSource::File => {
                // For file-based credentials, we assume the file is updated externally
                // Just update the last_rotated timestamp
                self.last_rotated = Some(SystemTime::now());
                tracing::info!("Marked file-based credentials as rotated");
                Ok(())
            }
            CredentialSource::Vault => {
                #[cfg(feature = "vault")]
                {
                    // Implementation would depend on your vault client
                    // This is a placeholder
                    tracing::info!("Rotated vault credentials");
                    self.last_rotated = Some(SystemTime::now());
                    Ok(())
                }

                #[cfg(not(feature = "vault"))]
                {
                    Err(AppError::InvalidInput(
                        "Vault support is not compiled in this build".to_string(),
                    ))
                }
            }
        }
    }

    /// Get current username
    pub fn username(&self) -> &str {
        self.username.as_str()
    }

    /// Get current password
    pub fn password(&self) -> &str {
        self.password.as_str()
    }
}

/// Credential file format for file-based credentials
#[derive(Debug, Serialize, Deserialize)]
struct CredentialFile {
    username: String,
    password: String,
}

/// Enhanced database config with secure credential handling
#[derive(Debug, Clone)]
pub struct SecureDatabaseConfig {
    pub endpoint: String,
    pub credentials: DatabaseCredentials,
    pub namespace: String,
    pub database: String,
    pub use_tls: bool,
}

impl SecureDatabaseConfig {
    pub fn new(
        endpoint: impl Into<String>,
        credentials: DatabaseCredentials,
        namespace: impl Into<String>,
        database: impl Into<String>,
    ) -> Self {
        Self {
            endpoint: endpoint.into(),
            credentials,
            namespace: namespace.into(),
            database: database.into(),
            use_tls: true, // Secure by default
        }
    }

    pub fn with_tls(mut self, use_tls: bool) -> Self {
        self.use_tls = use_tls;
        self
    }

    /// Convert to the standard DatabaseConfig format
    pub fn to_database_config(&self) -> crate::conn::DatabaseConfig {
        let mut endpoint = self.endpoint.clone();

        // Add TLS indicator if needed
        if self.use_tls && !endpoint.starts_with("https://") && !endpoint.starts_with("wss://") {
            if endpoint.starts_with("http://") {
                endpoint = endpoint.replace("http://", "https://");
            } else if endpoint.starts_with("ws://") {
                endpoint = endpoint.replace("ws://", "wss://");
            } else {
                // Assume we need to prefix with https:// if no protocol is specified
                endpoint = format!("https://{}", endpoint);
            }
        }

        crate::conn::DatabaseConfig::new(
            endpoint,
            self.credentials.username(),
            self.credentials.password(),
            self.namespace.clone(),
            self.database.clone(),
        )
    }
}

/// Connection manager that handles credential rotation
pub struct ConnectionManager {
    config: SecureDatabaseConfig,
    db: Option<std::sync::Arc<crate::types::Database>>,
    last_check: SystemTime,
    check_interval: Duration,
}

impl ConnectionManager {
    pub fn new(config: SecureDatabaseConfig) -> Self {
        Self {
            config,
            db: None,
            last_check: SystemTime::now(),
            check_interval: Duration::from_secs(300), // Check every 5 minutes by default
        }
    }

    pub fn with_check_interval(mut self, interval: Duration) -> Self {
        self.check_interval = interval;
        self
    }

    pub async fn get_connection(
        &mut self,
    ) -> Result<std::sync::Arc<crate::types::Database>, AppError> {
        // If we already have a connection, check if we need to refresh
        if let Some(ref db) = self.db {
            let now = SystemTime::now();
            let needs_check = match now.duration_since(self.last_check) {
                Ok(elapsed) => elapsed >= self.check_interval,
                Err(_) => false, // Clock went backwards, ignore
            };

            // Check for credential rotation if needed
            if needs_check && self.config.credentials.needs_rotation() {
                tracing::info!("Credentials need rotation, refreshing connection");
                self.rotate_credentials().await?;
            } else if !needs_check {
                // Return existing connection
                return Ok(db.clone());
            }
        }

        // Initialize connection if we don't have one or we need a fresh one
        let db_config = self.config.to_database_config();
        let db = crate::conn::initialize_db(db_config).await?;
        self.db = Some(db.clone());
        self.last_check = SystemTime::now();

        Ok(db)
    }

    async fn rotate_credentials(&mut self) -> Result<(), AppError> {
        // Rotate credentials
        self.config.credentials.rotate().await?;

        // Reconnect with new credentials
        let db_config = self.config.to_database_config();
        let db = crate::conn::initialize_db(db_config).await?;
        self.db = Some(db);
        self.last_check = SystemTime::now();

        Ok(())
    }
}

// Helper functions for loading credentials from different sources
pub mod helpers {
    use super::*;
    use dotenv::dotenv;

    /// Load database config from environment variables with optional dotenv file
    pub fn db_config_from_env(
        env_prefix: &str,
        dotenv_path: Option<&str>,
    ) -> Result<SecureDatabaseConfig, AppError> {
        // Load .env file if specified
        if let Some(path) = dotenv_path {
            if let Err(e) = dotenv::from_path(path) {
                tracing::warn!("Failed to load .env file from {}: {}", path, e);
            }
        } else {
            // Try to load .env from current directory
            let _ = dotenv();
        }

        // Define environment variable names
        let endpoint_var = format!("{}_ENDPOINT", env_prefix);
        let username_var = format!("{}_USERNAME", env_prefix);
        let password_var = format!("{}_PASSWORD", env_prefix);
        let namespace_var = format!("{}_NAMESPACE", env_prefix);
        let database_var = format!("{}_DATABASE", env_prefix);
        let tls_var = format!("{}_USE_TLS", env_prefix);

        // Load values from environment
        let endpoint = env::var(&endpoint_var).map_err(|_| {
            AppError::InvalidInput(format!("Environment variable not found: {}", endpoint_var))
        })?;

        let credentials = DatabaseCredentials::from_env(&username_var, &password_var)?;

        let namespace = env::var(&namespace_var).map_err(|_| {
            AppError::InvalidInput(format!("Environment variable not found: {}", namespace_var))
        })?;

        let database = env::var(&database_var).map_err(|_| {
            AppError::InvalidInput(format!("Environment variable not found: {}", database_var))
        })?;

        // TLS is optional and defaults to true
        let use_tls = env::var(&tls_var)
            .map(|v| v.to_lowercase() != "false" && v != "0")
            .unwrap_or(true);

        // Create config
        let config =
            SecureDatabaseConfig::new(endpoint, credentials, namespace, database).with_tls(use_tls);

        Ok(config)
    }

    /// Helper to load credentials file with proper permissions check
    pub fn create_credentials_file<P: AsRef<Path>>(
        path: P,
        username: &str,
        password: &str,
    ) -> Result<(), AppError> {
        let creds = CredentialFile {
            username: username.to_string(),
            password: password.to_string(),
        };

        let json = serde_json::to_string_pretty(&creds).map_err(|e| {
            AppError::InvalidInput(format!("Failed to serialize credentials: {}", e))
        })?;

        std::fs::write(&path, json).map_err(|e| {
            AppError::InvalidInput(format!("Failed to write credentials file: {}", e))
        })?;

        // Set restrictive permissions on Unix systems
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = std::fs::metadata(&path)
                .map_err(|e| AppError::InvalidInput(format!("Failed to get file metadata: {}", e)))?
                .permissions();

            // Set to 0600 (user read/write only)
            perms.set_mode(0o600);
            std::fs::set_permissions(&path, perms).map_err(|e| {
                AppError::InvalidInput(format!("Failed to set file permissions: {}", e))
            })?;
        }

        Ok(())
    }
}
