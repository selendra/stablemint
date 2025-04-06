use app_error::{AppError, AppResult};
use lazy_static::lazy_static;
use regex::Regex;

lazy_static! {
    // Email validation regex
    // This pattern checks for a valid email format with proper domain
    static ref EMAIL_REGEX: Regex = Regex::new(
        r"^([a-z0-9_+]([a-z0-9_+.]*[a-z0-9_+])?)@([a-z0-9]+([\-\.]{1}[a-z0-9]+)*\.[a-z]{2,6})"
    ).unwrap();

    // Username validation regex
    // Alphanumeric characters, underscores, and hyphens, 3-30 characters
    static ref USERNAME_REGEX: Regex = Regex::new(
        r"^[a-zA-Z0-9_-]{3,30}$"
    ).unwrap();

    // Password strength regex
    // At least 8 characters, must contain at least one uppercase letter,
    // one lowercase letter, one number, and one special character
    static ref STRONG_PASSWORD_REGEX: Regex = Regex::new(
        r"^(?=.*[a-z])(?=.*[A-Z])(?=.*\d)(?=.*[@$!%*?&])[A-Za-z\d@$!%*?&]{8,}$"
    ).unwrap();
}

/// Validates a username
pub fn validate_username(username: &str) -> AppResult<()> {
    if username.trim().is_empty() {
        return Err(AppError::ValidationError(
            "Username cannot be empty".to_string(),
        ));
    }

    if !USERNAME_REGEX.is_match(username) {
        return Err(AppError::ValidationError(
            "Username must be 3-30 characters long and can only contain letters, numbers, underscores, and hyphens".to_string()
        ));
    }

    Ok(())
}

/// Validates an email address
pub fn validate_email(email: &str) -> AppResult<()> {
    if email.trim().is_empty() {
        return Err(AppError::ValidationError(
            "Email cannot be empty".to_string(),
        ));
    }

    if !EMAIL_REGEX.is_match(email) {
        return Err(AppError::ValidationError(
            "Invalid email format".to_string(),
        ));
    }

    Ok(())
}

/// Validates a name
pub fn validate_name(name: &str) -> AppResult<()> {
    if name.trim().is_empty() {
        return Err(AppError::ValidationError(
            "Name cannot be empty".to_string(),
        ));
    }

    if name.trim().len() < 2 {
        return Err(AppError::ValidationError(
            "Name must be at least 2 characters long".to_string(),
        ));
    }

    if name.trim().len() > 100 {
        return Err(AppError::ValidationError(
            "Name cannot exceed 100 characters".to_string(),
        ));
    }

    Ok(())
}

pub fn validate_password(password: &str) -> AppResult<()> {
    if password.trim().is_empty() {
        return Err(AppError::ValidationError(
            "Password cannot be empty".to_string(),
        ));
    }

    if password.len() < 8 {
        return Err(AppError::ValidationError(
            "Password must be at least 8 characters long".to_string(),
        ));
    }

    let has_lowercase = password.chars().any(|c| c.is_ascii_lowercase());
    let has_uppercase = password.chars().any(|c| c.is_ascii_uppercase());
    let has_digit = password.chars().any(|c| c.is_ascii_digit());
    let has_special = password
        .chars()
        .any(|c| matches!(c, '@' | '$' | '!' | '%' | '*' | '?' | '&'));

    if !has_lowercase || !has_uppercase || !has_digit || !has_special {
        return Err(AppError::ValidationError(
            "Password must contain at least one uppercase letter, one lowercase letter, one number, and one special character (@$!%*?&)".to_string()
        ));
    }

    Ok(())
}

/// Sanitizes a string input by trimming whitespace
pub fn sanitize_string(input: &str) -> String {
    input.trim().to_string()
}
