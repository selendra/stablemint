use async_graphql::Error as GraphQLError;
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
        }
    }
}

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

// Response structure for API error responses
#[derive(Serialize, Deserialize)]
pub struct ErrorResponse {
    pub status: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<String>,
}

// Convert AppError to Axum HTTP Response
impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, error_message) = match &self {
            Self::ConfigError(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Configuration error"),
            Self::DatabaseError(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Database error"),
            Self::GraphQLError(_) => (StatusCode::BAD_REQUEST, "GraphQL processing error"),
            Self::ServerError(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error"),
            Self::ValidationError(_) => (StatusCode::BAD_REQUEST, "Validation error"),
            Self::NotFoundError(_) => (StatusCode::NOT_FOUND, "Resource not found"),
            Self::AuthenticationError(_) => (StatusCode::UNAUTHORIZED, "Authentication required"),
            Self::AuthorizationError(_) => (StatusCode::FORBIDDEN, "Access denied"),
        };

        // Log the full error
        tracing::error!("Error: {}", self);

        // Return a clean response to the client
        let body = Json(ErrorResponse {
            status: status.to_string(),
            message: error_message.to_string(),
            details: if status == StatusCode::INTERNAL_SERVER_ERROR {
                None // Don't expose internal error details to clients
            } else {
                Some(self.to_string())
            },
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
