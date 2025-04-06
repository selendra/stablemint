use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::{fs, path::Path};
use tracing::{debug, info, warn};

use app_error::{AppError, AppResult};

/// Complete application configuration loaded from JSON file
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AppConfig {
    pub environment: String,
    pub database: DatabaseConfig,
    pub server: ServerConfig,
    pub security: SecurityConfig,
    pub monitoring: MonitoringConfig,
    pub bodylimit: BodyLimitConfig,
    pub redis: Option<RedisConfig>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DatabaseConfig {
    pub endpoint: String,
    pub username: String,
    pub password: String,
    pub namespace: String,
    pub database: String,
    pub pool: DbPoolConfig,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DbPoolConfig {
    pub size: usize,
    pub connection_timeout: u64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub timeouts: ServerTimeouts,
    pub body_limit: usize,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ServerTimeouts {
    pub read: u64,
    pub write: u64,
    pub idle: u64,
    pub keep_alive: u64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SecurityConfig {
    pub jwt: JwtConfig,
    pub cors: CorsConfig,
    pub rate_limiting: RateLimitingConfig,
    pub password: PasswordConfig,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct JwtConfig {
    pub secret: String,
    pub expiry_hours: u64,
    pub algorithm: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CorsConfig {
    pub allowed_origins: Vec<String>,
    pub allowed_methods: Vec<String>,
    pub allowed_headers: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RateLimitingConfig {
    pub api: RateLimitSettings,
    pub login: RateLimitSettings,
    pub paths: std::collections::HashMap<String, usize>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RateLimitSettings {
    pub max_attempts: usize,
    pub window_duration: u64,
    pub block_duration: Option<u64>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PasswordConfig {
    pub min_length: usize,
    pub require_uppercase: bool,
    pub require_lowercase: bool,
    pub require_number: bool,
    pub require_special: bool,
    pub argon2: Argon2Config,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Argon2Config {
    pub variant: String,
    pub memory: u32,
    pub iterations: u32,
    pub parallelism: u32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MonitoringConfig {
    pub sentry: SentryConfig,
    pub logging: LoggingConfig,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SentryConfig {
    pub dsn: String,
    pub sample_rate: f32,
    pub traces_sample_rate: f32,
    pub environment: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LoggingConfig {
    pub level: String,
    pub format: String,
    pub hide_secrets: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BodyLimitConfig {
    pub user: usize,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RedisConfig {
    pub url: String,
    pub pool_size: usize,
    pub connection_timeout: u64,
    pub prefix: Option<String>,
}


impl AppConfig {
    /// Load configuration from a JSON file
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let config_content = fs::read_to_string(path)?;
        let config: AppConfig = serde_json::from_str(&config_content)?;
        
        debug!("Configuration loaded from file");
        
        Ok(config)
    }
    
    /// Load configuration from the default location
    pub fn load() -> AppResult<Self> {
        let config_content = std::str::from_utf8(include_bytes!("../res/app-config.json")).expect("Invalid UTF-8");
        // Try to load the config from file
        let config = match serde_json::from_str::<AppConfig>(&config_content) {
            Ok(conf) => {
                info!("Loaded configuration from: {:?}", conf.environment);
                conf
            },
            Err(e) => {
                warn!("Failed to load config file: {}. Using default configuration.", e);
                Self::default()
            }
        };
        
        // Validate the config
        config.validate()?;
        
        Ok(config)
    }
    
    /// Validate the configuration
    pub fn validate(&self) -> AppResult<()> {
        let mut errors = Vec::new();
        
        // Only perform strict validation in production
        let is_production = self.environment == "production";
        
        // Validate database configuration
        if self.database.endpoint.trim().is_empty() {
            errors.push("Database endpoint cannot be empty".to_string());
        } else if is_production && !self.database.endpoint.starts_with("wss://") && 
                  !self.database.endpoint.contains("memory") {
            errors.push("Production should use a secure 'wss://' database connection".to_string());
        }
        
        if self.database.namespace.trim().is_empty() {
            errors.push("Database namespace cannot be empty".to_string());
        }
        
        if self.database.database.trim().is_empty() {
            errors.push("Database name cannot be empty".to_string());
        }
        
        if is_production && self.database.username == "root" {
            errors.push("Using default 'root' username in production is insecure".to_string());
        }
        
        if is_production && self.database.password == "root" {
            errors.push("Using default 'root' password in production is insecure".to_string());
        }
        
        // Validate server configuration
        if self.server.host.trim().is_empty() {
            errors.push("Server host cannot be empty".to_string());
        }
        
        if self.server.port == 0 {
            errors.push("Server port cannot be 0".to_string());
        }
        
        // Validate security configuration
        if is_production && (self.security.jwt.secret.len() < 32 || 
                            self.security.jwt.secret == "your-strong-secret-key-here") {
            errors.push("JWT secret is not secure for production use".to_string());
        }
        
        // Validate monitoring configuration
        if is_production && self.monitoring.sentry.dsn.trim().is_empty() {
            errors.push("Sentry DSN should be configured in production".to_string());
        }
        
            // Validate Redis configuration if present
        if let Some(ref redis_config) = self.redis {
            if redis_config.url.trim().is_empty() {
                errors.push("Redis URL cannot be empty".to_string());
            } else if self.environment == "production" && 
                    !redis_config.url.starts_with("rediss://") {
                errors.push("Production should use a secure 'rediss://' Redis connection".to_string());
            }
            
            if redis_config.pool_size == 0 {
                errors.push("Redis pool size must be greater than 0".to_string());
            }
        }
        
        if !errors.is_empty() {
            return Err(AppError::ConfigError(anyhow::anyhow!(
                "Invalid configuration: {}",
                errors.join(", ")
            )));
        }
            Ok(())
    }
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            environment: "development".to_string(),
            database: DatabaseConfig {
                endpoint: "ws://localhost:8000".to_string(),
                username: "root".to_string(),
                password: "root".to_string(),
                namespace: "selendraDb".to_string(),
                database: "cryptoBank".to_string(),
                pool: DbPoolConfig {
                    size: 10,
                    connection_timeout: 5000,
                },
            },
            server: ServerConfig {
                host: "0.0.0.0".to_string(),
                port: 3000,
                timeouts: ServerTimeouts {
                    read: 30000,
                    write: 30000,
                    idle: 60000,
                    keep_alive: 15000,
                },
                body_limit: 1048576, // 1MB
            },
            security: SecurityConfig {
                jwt: JwtConfig {
                    secret: "default-insecure-jwt-secret-do-not-use-in-production".to_string(),
                    expiry_hours: 24,
                    algorithm: "HS256".to_string(),
                },
                cors: CorsConfig {
                    allowed_origins: vec!["*".to_string()],
                    allowed_methods: vec!["GET".to_string(), "POST".to_string(), "OPTIONS".to_string()],
                    allowed_headers: vec!["Content-Type".to_string(), "Authorization".to_string()],
                },
                rate_limiting: RateLimitingConfig {
                    api: RateLimitSettings {
                        max_attempts: 100,
                        window_duration: 60,
                        block_duration: None,
                    },
                    login: RateLimitSettings {
                        max_attempts: 5,
                        window_duration: 300,
                        block_duration: Some(900),
                    },
                    paths: std::collections::HashMap::new(),
                },
                password: PasswordConfig {
                    min_length: 8,
                    require_uppercase: true,
                    require_lowercase: true,
                    require_number: true,
                    require_special: true,
                    argon2: Argon2Config {
                        variant: "argon2id".to_string(),
                        memory: 65536,
                        iterations: 3,
                        parallelism: 4,
                    },
                },
            },
            monitoring: MonitoringConfig {
                sentry: SentryConfig {
                    dsn: "".to_string(),
                    sample_rate: 1.0,
                    traces_sample_rate: 0.2,
                    environment: "development".to_string(),
                },
                logging: LoggingConfig {
                    level: "info".to_string(),
                    format: "json".to_string(),
                    hide_secrets: true,
                },
            },
            bodylimit: BodyLimitConfig {
                user: 1048576, // 1MB
            },
            redis: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;
    use tempfile::tempdir;
    
    #[test]
    fn test_config_from_file() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test-config.json");
        
        let config_json = r#"{
            "environment": "test",
            "database": {
                "endpoint": "ws://test-db:8000",
                "username": "test_user",
                "password": "test_pass",
                "namespace": "test_ns",
                "database": "test_db",
                "pool": {
                    "size": 5,
                    "connection_timeout": 3000
                }
            },
            "server": {
                "host": "127.0.0.1",
                "port": 4000,
                "timeouts": {
                    "read": 20000,
                    "write": 20000,
                    "idle": 40000,
                    "keep_alive": 10000
                },
                "body_limit": 2097152
            },
            "security": {
                "jwt": {
                    "secret": "test-secret-key",
                    "expiry_hours": 12,
                    "algorithm": "HS256"
                },
                "cors": {
                    "allowed_origins": ["http://localhost:3000"],
                    "allowed_methods": ["GET", "POST"],
                    "allowed_headers": ["Content-Type"]
                },
                "rate_limiting": {
                    "api": {
                        "max_attempts": 50,
                        "window_duration": 30,
                        "block_duration": null
                    },
                    "login": {
                        "max_attempts": 3,
                        "window_duration": 150,
                        "block_duration": 300
                    },
                    "paths": {
                        "/test": 5
                    }
                },
                "password": {
                    "min_length": 10,
                    "require_uppercase": true,
                    "require_lowercase": true,
                    "require_number": true,
                    "require_special": true,
                    "argon2": {
                        "variant": "argon2id",
                        "memory": 32768,
                        "iterations": 2,
                        "parallelism": 2
                    }
                }
            },
            "monitoring": {
                "sentry": {
                    "dsn": "https://test-dsn@sentry.io/123",
                    "sample_rate": 0.5,
                    "traces_sample_rate": 0.1,
                    "environment": "test"
                },
                "logging": {
                    "level": "debug",
                    "format": "text",
                    "hide_secrets": true
                }
            },
            "bodylimit": {
                "user": 1048576
            }
        }"#;
        
        // Write test config to temp file
        let mut file = File::create(&file_path).unwrap();
        file.write_all(config_json.as_bytes()).unwrap();
        
        // Load the config
        let config = AppConfig::from_file(&file_path).unwrap();

        println!("config: {:?}", config);
        
        // Verify loaded values
        assert_eq!(config.environment, "test");
        assert_eq!(config.database.endpoint, "ws://test-db:8000");
        assert_eq!(config.database.username, "test_user");
        assert_eq!(config.server.port, 4000);
        assert_eq!(config.security.jwt.expiry_hours, 12);
        assert_eq!(config.monitoring.logging.level, "debug");
        
        // Verify nested values
        assert_eq!(config.database.pool.size, 5);
        assert_eq!(config.security.rate_limiting.login.max_attempts, 3);
        assert_eq!(config.security.password.min_length, 10);
        
        // Verify collections
        assert_eq!(config.security.cors.allowed_origins.len(), 1);
        assert_eq!(config.security.cors.allowed_origins[0], "http://localhost:3000");
        assert_eq!(config.security.rate_limiting.paths.get("/test"), Some(&5));
    }
    
    #[test]
    fn test_config_validation() {
        // Valid config
        let config = AppConfig::default();
        assert!(config.validate().is_ok());
        
        // Invalid config (production with insecure settings)
        let mut prod_config = AppConfig::default();
        prod_config.environment = "production".to_string();
        
        // Should fail validation in production
        assert!(prod_config.validate().is_err());
        
        // Fix the config
        prod_config.database.endpoint = "wss://secure-db.example.com".to_string();
        prod_config.database.username = "secure_user".to_string();
        prod_config.database.password = "secure_password".to_string();
        prod_config.security.jwt.secret = "a-very-secure-and-long-jwt-secret-key-for-production-use".to_string();
        prod_config.monitoring.sentry.dsn = "https://exampledsn@sentry.io/123456".to_string();
        
        // Should pass validation now
        assert!(prod_config.validate().is_ok());
    }
}