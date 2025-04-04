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

impl AppError {
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
            Self::RateLimitError(msg) => write!(f, "RateLimitError error: {}", msg),
            Self::InputError(msg) => write!(f, "InputError error: {}", msg),
            Self::CryptoError(msg) => write!(f, "CryptoError error: {}", msg),
            Self::NetworkError(msg) => write!(f, "NetworkError error: {}", msg),
            Self::ResourceExistsError(msg) => write!(f, "ResourceExistsError error: {}", msg),
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct ErrorResponse {
    pub status: String,
    pub message: String,
    pub code: String, // Add an error code field
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub help: Option<String>, // Add suggested actions
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, error_message, error_code, help_text) = match &self {
            Self::ConfigError(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "System configuration error",
                "CONFIG_ERROR",
                None,
            ),
            Self::DatabaseError(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Database operation failed",
                "DB_ERROR",
                None,
            ),
            Self::ValidationError(msg) => (
                StatusCode::BAD_REQUEST,
                msg.as_str(),
                "VALIDATION_ERROR",
                Some("Please review your input and try again."),
            ),
            Self::NotFoundError(msg) => (
                StatusCode::NOT_FOUND,
                msg.as_str(),
                "NOT_FOUND",
                Some("The requested resource was not found."),
            ),
            Self::AuthenticationError(msg) => (
                StatusCode::UNAUTHORIZED,
                msg.as_str(),
                "AUTH_ERROR",
                Some("Please log in to access this resource."),
            ),
            Self::AuthorizationError(msg) => (
                StatusCode::FORBIDDEN,
                msg.as_str(),
                "FORBIDDEN",
                Some("You don't have permission to access this resource."),
            ),
            Self::RateLimitError(msg) => (
                StatusCode::TOO_MANY_REQUESTS,
                msg.as_str(),
                "RATE_LIMIT",
                Some("Please try again later."),
            ),
            // Handle other error types...
            _ => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Internal server error",
                "SERVER_ERROR",
                None,
            ),
        };

        // Log the error with context
        let log_message = format!("[{}] {}: {}", error_code, status, self);
        if status.is_server_error() {
            tracing::error!(error_code = error_code, status_code = %status.as_u16(), %error_message, "{}", log_message);
        } else {
            tracing::warn!(error_code = error_code, status_code = %status.as_u16(), %error_message, "{}", log_message);
        }

        // Return a clean response to the client
        let body = Json(ErrorResponse {
            status: status.to_string(),
            message: error_message.to_string(),
            code: error_code.to_string(),
            details: if status == StatusCode::INTERNAL_SERVER_ERROR {
                None // Don't expose internal error details to clients
            } else {
                Some(self.to_string())
            },
            help: help_text.map(String::from),
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
        let mut error = FieldError::new(self.to_string());

        // Add appropriate extensions based on error type
        match self {
            Self::ValidationError(msg) => {
                error = error.extend_with(|_, e| {
                    e.set("code", "VALIDATION_ERROR");
                    e.set("details", msg);
                });
            }
            Self::AuthenticationError(msg) => {
                error = error.extend_with(|_, e| {
                    e.set("code", "AUTHENTICATION_ERROR");
                    e.set("details", msg);
                });
            }
            Self::AuthorizationError(msg) => {
                error = error.extend_with(|_, e| {
                    e.set("code", "AUTHORIZATION_ERROR");
                    e.set("details", msg);
                });
            }
            Self::NotFoundError(msg) => {
                error = error.extend_with(|_, e| {
                    e.set("code", "NOT_FOUND_ERROR");
                    e.set("details", msg);
                });
            }
            Self::DatabaseError(_) => {
                error = error.extend_with(|_, e| {
                    e.set("code", "DATABASE_ERROR");
                    e.set("details", "A database error occurred");
                });
            }
            Self::ConfigError(_) => {
                error = error.extend_with(|_, e| {
                    e.set("code", "CONFIG_ERROR");
                    e.set("details", "A configuration error occurred");
                });
            }
            Self::ServerError(_) => {
                error = error.extend_with(|_, e| {
                    e.set("code", "SERVER_ERROR");
                    e.set("details", "An internal server error occurred");
                });
            }
            Self::GraphQLError(err) => {
                error = error.extend_with(|_, e| {
                    e.set("code", "GRAPHQL_ERROR");
                    e.set("details", format!("{:?}", err));
                });
            }
            // If you've added additional error types
            _ => {
                error = error.extend_with(|_, e| {
                    e.set("code", "UNKNOWN_ERROR");
                    e.set("details", "An unknown error occurred");
                });
            }
        };

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
