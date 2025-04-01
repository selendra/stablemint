use stablemint_error::AppError;
use tracing::{error, info, warn};

pub struct ErrorHandler;

impl ErrorHandler {
    // Log errors with appropriate severity and return safe messages
    pub fn handle(error: AppError) -> (String, Option<i32>) {
        match &error {
            AppError::Database(e) => {
                error!(error_code = error.error_code(), error = %e, "Database error");
                (error.user_message(), Some(500))
            }

            AppError::ConnectionError(e) => {
                error!(error_code = error.error_code(), error = %e, "Database connection error");
                (error.user_message(), Some(500))
            }
            AppError::QueryError(e) => {
                error!(error_code = error.error_code(), error = %e, "Database query error");
                (error.user_message(), Some(500))
            }
            AppError::AuthError(msg) => {
                warn!(error_code = error.error_code(), message = %msg, "Authentication error");
                (error.user_message(), Some(401))
            }
            AppError::AccessDenied(msg) => {
                warn!(error_code = error.error_code(), message = %msg, "Authorization error");
                (error.user_message(), Some(403))
            }
            AppError::NotFound => {
                info!(error_code = error.error_code(), "Resource not found");
                (error.user_message(), Some(404))
            }
            AppError::InvalidInput(msg) => {
                info!(error_code = error.error_code(), message = %msg, "Invalid input");
                (error.user_message(), Some(400))
            }
            AppError::ConfigError(msg) => {
                error!(error_code = error.error_code(), message = %msg, "Configuration error");
                (error.user_message(), Some(500))
            }
            AppError::CredentialError(msg) => {
                error!(error_code = error.error_code(), message = %msg, "Credential error");
                (error.user_message(), Some(500))
            }
            AppError::Internal(e) => {
                error!(error_code = error.error_code(), error = %e, "Internal server error");
                (error.user_message(), Some(500))
            }
        }
    }
}
