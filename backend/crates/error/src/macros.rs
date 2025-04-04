/// Provides a convenient way to add context to errors
/// 
/// # Example
/// ```
/// with_context!(db_operation, "Failed to fetch user data")
/// ```
#[macro_export]
macro_rules! with_context {
    ($result:expr, $context:expr) => {
        $result.map_err(|e| {
            tracing::error!("{}: {}", $context, e);
            app_error::AppError::DatabaseError(anyhow::anyhow!("{}: {}", $context, e))
        })
    };
    
    ($result:expr, $error_type:ident, $context:expr) => {
        $result.map_err(|e| {
            tracing::error!("{}: {}", $context, e);
            app_error::AppError::$error_type(anyhow::anyhow!("{}: {}", $context, e))
        })
    };
}

/// Simplifies creating validation errors
/// 
/// # Example
/// ```
/// validation_error!("username", "Username must be at least 3 characters long")
/// ```
#[macro_export]
macro_rules! validation_error {
    ($field:expr, $message:expr) => {
        Err(app_error::AppError::ValidationError(
            format!("Validation failed for '{}': {}", $field, $message)
        ))
    };
}

/// Simplifies creating not found errors
/// 
/// # Example
/// ```
/// not_found_error!("User", user_id)
/// ```
#[macro_export]
macro_rules! not_found_error {
    ($resource_type:expr, $identifier:expr) => {
        Err(app_error::AppError::NotFoundError(
            format!("{} with identifier '{}' was not found.", $resource_type, $identifier)
        ))
    };
}

/// Simplifies creating resource exists errors
/// 
/// # Example
/// ```
/// resource_exists_error!("User", "username", username)
/// ```
#[macro_export]
macro_rules! resource_exists_error {
    ($resource_type:expr, $field:expr, $value:expr) => {
        Err(app_error::AppError::ResourceExistsError(
            format!("{} with {} '{}' already exists.", $resource_type, $field, $value)
        ))
    };
}

/// Simplifies creating authentication errors
/// 
/// # Example
/// ```
/// auth_error!("Invalid username or password")
/// ```
#[macro_export]
macro_rules! auth_error {
    ($message:expr) => {
        Err(app_error::AppError::AuthenticationError($message.to_string()))
    };
}