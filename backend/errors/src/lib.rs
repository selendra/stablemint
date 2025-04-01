// Application error types with improved categorization
#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("Database error: {0}")]
    Database(anyhow::Error),

    #[error("Database connection error: {0}")]
    ConnectionError(anyhow::Error),

    #[error("Database query error: {0}")]
    QueryError(anyhow::Error),

    #[error("Authentication error: {0}")]
    AuthError(String),

    #[error("Authorization error: {0}")]
    AccessDenied(String),

    #[error("Resource not found")]
    NotFound,

    #[error("Invalid input: {0}")]
    InvalidInput(String),

    #[error("Configuration error: {0}")]
    ConfigError(String),

    #[error("Credential error: {0}")]
    CredentialError(String),

    #[error("Internal server error")]
    Internal(anyhow::Error),
}

impl From<anyhow::Error> for AppError {
    fn from(error: anyhow::Error) -> Self {
        AppError::Internal(error)
    }
}

// Add implementation for security-safe error messages
impl AppError {
    // Get a user-safe error message that doesn't expose sensitive details
    pub fn user_message(&self) -> String {
        match self {
            AppError::Database(_) => "Database operation failed".to_string(),
            AppError::ConnectionError(_) => "Database connection failed".to_string(),
            AppError::QueryError(_) => "Database operation failed".to_string(),
            AppError::AuthError(_) => "Authentication failed".to_string(),
            AppError::AccessDenied(_) => "Access denied".to_string(),
            AppError::NotFound => "Resource not found".to_string(),
            AppError::InvalidInput(msg) => format!("Invalid input: {}", msg),
            AppError::ConfigError(_) => "System configuration error".to_string(),
            AppError::CredentialError(_) => "Credential error".to_string(),
            AppError::Internal(_) => "An internal error occurred".to_string(),
        }
    }

    // Get error code for logging/tracking
    pub fn error_code(&self) -> &'static str {
        match self {
            AppError::Database(_) => "DB_ERR_001",
            AppError::ConnectionError(_) => "DB_CONN_001",
            AppError::QueryError(_) => "DB_QUERY_001",
            AppError::AuthError(_) => "AUTH_001",
            AppError::AccessDenied(_) => "ACCESS_001",
            AppError::NotFound => "RESOURCE_001",
            AppError::InvalidInput(_) => "INPUT_001",
            AppError::ConfigError(_) => "CONFIG_001",
            AppError::CredentialError(_) => "CRED_001",
            AppError::Internal(_) => "INTERNAL_001",
        }
    }
}
