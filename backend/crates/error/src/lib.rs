pub mod middleware_handling;

use async_graphql::{Error as GraphQLError, ErrorExtensions, FieldError};
use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug)]
pub enum AppError {
    ConfigError(anyhow::Error),
    DatabaseError(anyhow::Error),
    GraphQLError(GraphQLError),
    ServerError(anyhow::Error),
    ValidationError(String),
    NotFoundError(String),
    AuthenticationError(String),
    AuthorizationError(String),
    RateLimitError(String),
    InputError(String),
    CryptoError(String),
    NetworkError(String),
    ResourceExistsError(String),
}

// Mapping between error types and HTTP status codes/messages
// Extracting this to a constant avoids duplication
const ERROR_MAPPINGS: &[(&str, StatusCode, &str, &str, Option<&str>)] = &[
    ("ConfigError", StatusCode::INTERNAL_SERVER_ERROR, "CONFIG_ERROR", "System configuration error", None),
    ("DatabaseError", StatusCode::INTERNAL_SERVER_ERROR, "DB_ERROR", "Database operation failed", None),
    ("ValidationError", StatusCode::BAD_REQUEST, "VALIDATION_ERROR", "", Some("Please review your input and try again.")),
    ("NotFoundError", StatusCode::NOT_FOUND, "NOT_FOUND", "", Some("The requested resource was not found.")),
    ("AuthenticationError", StatusCode::UNAUTHORIZED, "AUTH_ERROR", "", Some("Please log in to access this resource.")),
    ("AuthorizationError", StatusCode::FORBIDDEN, "FORBIDDEN", "", Some("You don't have permission to access this resource.")),
    ("RateLimitError", StatusCode::TOO_MANY_REQUESTS, "RATE_LIMIT", "", Some("Please try again later.")),
    ("InputError", StatusCode::BAD_REQUEST, "INPUT_ERROR", "", Some("Invalid input provided.")),
    ("CryptoError", StatusCode::INTERNAL_SERVER_ERROR, "CRYPTO_ERROR", "Encryption error", None),
    ("NetworkError", StatusCode::SERVICE_UNAVAILABLE, "NETWORK_ERROR", "Network error", None),
    ("ResourceExistsError", StatusCode::CONFLICT, "RESOURCE_EXISTS", "", Some("The resource already exists.")),
    // Default case for ServerError and others
    ("", StatusCode::INTERNAL_SERVER_ERROR, "SERVER_ERROR", "Internal server error", None),
];

impl AppError {
    // Helper to get the error type name as a string
    fn error_type_name(&self) -> &str {
        match self {
            Self::ConfigError(_) => "ConfigError",
            Self::DatabaseError(_) => "DatabaseError",
            Self::GraphQLError(_) => "GraphQLError",
            Self::ServerError(_) => "ServerError",
            Self::ValidationError(_) => "ValidationError",
            Self::NotFoundError(_) => "NotFoundError",
            Self::AuthenticationError(_) => "AuthenticationError",
            Self::AuthorizationError(_) => "AuthorizationError",
            Self::RateLimitError(_) => "RateLimitError",
            Self::InputError(_) => "InputError",
            Self::CryptoError(_) => "CryptoError",
            Self::NetworkError(_) => "NetworkError",
            Self::ResourceExistsError(_) => "ResourceExistsError",
        }
    }

    // User-friendly authentication errors
    pub fn invalid_credentials() -> Self {
        Self::AuthenticationError(
            "Invalid username or password. Please check your credentials and try again."
                .to_string(),
        )
    }

    pub fn account_locked(seconds: u64) -> Self {
        Self::RateLimitError(format!(
            "Your account has been temporarily locked for security. Please try again in {} seconds or reset your password.",
            seconds
        ))
    }

    pub fn token_expired() -> Self {
        Self::AuthenticationError(
            "Your session has expired. Please log in again to continue.".to_string(),
        )
    }

    pub fn token_invalid() -> Self {
        Self::AuthenticationError("Invalid authentication token. Please log in again.".to_string())
    }

    // Resource errors
    pub fn resource_not_found(resource_type: &str, identifier: &str) -> Self {
        Self::NotFoundError(format!(
            "{} with identifier '{}' was not found.",
            resource_type, identifier
        ))
    }

    pub fn resource_exists(resource_type: &str, identifier: &str) -> Self {
        Self::ResourceExistsError(format!(
            "{} with identifier '{}' already exists.",
            resource_type, identifier
        ))
    }

    // Validation errors
    pub fn validation(field: &str, message: &str) -> Self {
        Self::ValidationError(format!("Validation failed for '{}': {}", field, message))
    }

    // Database errors with user-friendly messages
    pub fn database_operation_failed(operation: &str, resource: &str) -> Self {
        Self::DatabaseError(anyhow::anyhow!(
            "Database operation '{}' failed on resource '{}'",
            operation,
            resource
        ))
    }
    
    // Helper to get error details based on error type
    fn get_error_details(&self) -> (StatusCode, String, String, Option<String>) {
        let error_type = self.error_type_name();
        
        // Find matching error mapping
        for &(err_type, status, code, default_msg, help) in ERROR_MAPPINGS {
            if err_type == error_type {
                let message = match self {
                    Self::ValidationError(msg) | 
                    Self::NotFoundError(msg) | 
                    Self::AuthenticationError(msg) | 
                    Self::AuthorizationError(msg) | 
                    Self::RateLimitError(msg) |
                    Self::InputError(msg) |
                    Self::CryptoError(msg) |
                    Self::NetworkError(msg) |
                    Self::ResourceExistsError(msg) => msg.clone(),
                    _ => default_msg.to_string(),
                };
                
                return (status, code.to_string(), message, help.map(String::from));
            }
        }
        
        // Default case
        let (_, status, code, default_msg, help) = ERROR_MAPPINGS.last().unwrap();
        (
            *status, 
            code.to_string(), 
            default_msg.to_string(), 
            help.map(String::from)
        )
    }
}

impl std::error::Error for AppError {}

// Convert from various error types to AppError
impl From<anyhow::Error> for AppError {
    fn from(error: anyhow::Error) -> Self {
        Self::ServerError(error)
    }
}

impl From<GraphQLError> for AppError {
    fn from(error: GraphQLError) -> Self {
        Self::GraphQLError(error)
    }
}

// Human-friendly error messages
impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ConfigError(e) => write!(f, "Configuration error: {}", e),
            Self::DatabaseError(e) => write!(f, "Database error: {}", e),
            Self::GraphQLError(e) => write!(f, "GraphQL error: {:?}", e),
            Self::ServerError(e) => write!(f, "Server error: {}", e),
            Self::ValidationError(msg) => write!(f, "Validation error: {}", msg),
            Self::NotFoundError(msg) => write!(f, "Not found: {}", msg),
            Self::AuthenticationError(msg) => write!(f, "Authentication error: {}", msg),
            Self::AuthorizationError(msg) => write!(f, "Authorization error: {}", msg),
            Self::RateLimitError(msg) => write!(f, "Rate limit error: {}", msg),
            Self::InputError(msg) => write!(f, "Input error: {}", msg),
            Self::CryptoError(msg) => write!(f, "Crypto error: {}", msg),
            Self::NetworkError(msg) => write!(f, "Network error: {}", msg),
            Self::ResourceExistsError(msg) => write!(f, "Resource exists error: {}", msg),
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct ErrorResponse {
    pub status: String,
    pub message: String,
    pub code: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub help: Option<String>,
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, error_code, error_message, help_text) = self.get_error_details();

        // Log the error with context
        let log_message = format!("[{}] {}: {}", error_code, status, self);
        if status.is_server_error() {
            tracing::error!(error_code = %error_code, status_code = %status.as_u16(), %error_message, "{}", log_message);
        } else {
            tracing::warn!(error_code = %error_code, status_code = %status.as_u16(), %error_message, "{}", log_message);
        }

        // Return a clean response to the client
        let body = Json(ErrorResponse {
            status: status.to_string(),
            message: error_message,
            code: error_code,
            details: if status == StatusCode::INTERNAL_SERVER_ERROR {
                None // Don't expose internal error details to clients
            } else {
                Some(self.to_string())
            },
            help: help_text,
        });

        (status, body).into_response()
    }
}

// Utility for anyhow results to AppError conversions
pub type AppResult<T> = Result<T, AppError>;

// Extension trait to wrap anyhow errors with specific context
pub trait AppErrorExt<T> {
    fn config_err(self) -> AppResult<T>;
    fn db_err(self) -> AppResult<T>;
    fn server_err(self) -> AppResult<T>;
}

impl<T, E> AppErrorExt<T> for Result<T, E>
where
    E: Into<anyhow::Error>,
{
    fn config_err(self) -> AppResult<T> {
        self.map_err(|e| AppError::ConfigError(e.into()))
    }

    fn db_err(self) -> AppResult<T> {
        self.map_err(|e| AppError::DatabaseError(e.into()))
    }

    fn server_err(self) -> AppResult<T> {
        self.map_err(|e| AppError::ServerError(e.into()))
    }
}

impl AppError {
    // Convert AppError to a GraphQL FieldError with appropriate extensions
    pub fn to_field_error(&self) -> FieldError {
        let (_, error_code, message, help) = self.get_error_details();
        
        let mut error = FieldError::new(message);

        // Add appropriate extensions based on error type
        error = error.extend_with(|_, e| {
            e.set("code", error_code);
            
            // Add help text if available
            if let Some(help_text) = help {
                e.set("help", help_text);
            }
            
            // Add detailed message for debugging
            match self {
                Self::ConfigError(err) | 
                Self::DatabaseError(err) | 
                Self::ServerError(err) => {
                    if cfg!(debug_assertions) {
                        e.set("details", format!("{:?}", err));
                    }
                },
                Self::ValidationError(msg) | 
                Self::NotFoundError(msg) | 
                Self::AuthenticationError(msg) | 
                Self::AuthorizationError(msg) |
                Self::RateLimitError(msg) |
                Self::InputError(msg) |
                Self::CryptoError(msg) |
                Self::NetworkError(msg) |
                Self::ResourceExistsError(msg) => {
                    e.set("details", msg);
                },
                Self::GraphQLError(err) => {
                    e.set("details", format!("{:?}", err));
                },
            }
        });

        // Log the error with appropriate level based on error type
        match self {
            Self::ServerError(_) | Self::DatabaseError(_) | Self::ConfigError(_) => {
                tracing::error!(error = %self, "GraphQL resolver error");
            }
            Self::AuthenticationError(_) | Self::AuthorizationError(_) => {
                tracing::warn!(error = %self, "Authentication/authorization error");
            }
            Self::ValidationError(_) | Self::NotFoundError(_) => {
                tracing::info!(error = %self, "Client request error");
            }
            _ => {
                tracing::warn!(error = %self, "GraphQL error");
            }
        }

        error
    }
}