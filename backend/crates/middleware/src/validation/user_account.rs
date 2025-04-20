// backend/crates/middleware/src/validation/user_account.rs
use app_config::AppConfig;
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

/// Validates password against configured requirements
pub fn validate_password(password: &str) -> AppResult<()> {
    // Load configuration (this handles errors gracefully and returns defaults if config can't be loaded)
    let config = AppConfig::load().unwrap_or_default();
    let password_config = &config.security.password;

    if password.trim().is_empty() {
        return Err(AppError::ValidationError(
            "Password cannot be empty".to_string(),
        ));
    }

    // Check minimum length requirement
    if password.len() < password_config.min_length {
        return Err(AppError::ValidationError(
            format!("Password must be at least {} characters long", password_config.min_length)
        ));
    }

    // Check for required character classes
    let has_lowercase = password.chars().any(|c| c.is_ascii_lowercase());
    let has_uppercase = password.chars().any(|c| c.is_ascii_uppercase());
    let has_digit = password.chars().any(|c| c.is_ascii_digit());
    let has_special = password
        .chars()
        .any(|c| matches!(c, '@' | '$' | '!' | '%' | '*' | '?' | '&' | '#' | '^' | '-' | '_' | '+' | '=' | '.' | ',' | ':' | ';'));

    // Validate according to configuration
    let mut missing = Vec::new();
    
    if password_config.require_lowercase && !has_lowercase {
        missing.push("lowercase letter");
    }
    
    if password_config.require_uppercase && !has_uppercase {
        missing.push("uppercase letter");
    }
    
    if password_config.require_number && !has_digit {
        missing.push("number");
    }
    
    if password_config.require_special && !has_special {
        missing.push("special character (@$!%*?&#^-_+=.,:;)");
    }

    if !missing.is_empty() {
        return Err(AppError::ValidationError(
            format!("Password must contain at least one {}", missing.join(", one "))
        ));
    }

    Ok(())
}

/// Sanitizes a string input by trimming whitespace
pub fn sanitize_string(input: &str) -> String {
    input.trim().to_string()
}


#[cfg(test)]
mod tests {
    use super::*;
    use app_config::{AppConfig, PasswordConfig, Argon2Config};

    #[test]
    fn test_config_based_password_validation() {
        // Create a test AppConfig with different password requirements
        let mut config = AppConfig::default();
        config.security.password = PasswordConfig {
            min_length: 10,
            require_uppercase: true,
            require_lowercase: true,
            require_number: true,
            require_special: true,
            argon2: Argon2Config {
                variant: "argon2id".to_string(),
                memory: 32768,
                iterations: 2,
                parallelism: 2,
            },
        };

        // Test with a password that meets all requirements
        let good_password = "StrongP@ss123";
        assert!(validate_password(good_password).is_ok(), 
            "Password should pass validation with the configured requirements");

        // Test with password that's too short
        let short_password = "Short@1";
        assert!(validate_password(short_password).is_err(), 
            "Password that's too short should fail validation");

        // Test with password that's missing uppercase
        let no_upper_password = "weakp@ssword123";
        assert!(validate_password(no_upper_password).is_err(), 
            "Password without uppercase should fail validation");

        // Test with password that's missing lowercase
        let no_lower_password = "STRONGP@SS123";
        assert!(validate_password(no_lower_password).is_err(), 
            "Password without lowercase should fail validation");

        // Test with password that's missing number
        let no_number_password = "StrongPassword@";
        assert!(validate_password(no_number_password).is_err(), 
            "Password without number should fail validation");

        // Test with password that's missing special character
        let no_special_password = "StrongPassword123";
        assert!(validate_password(no_special_password).is_err(), 
            "Password without special character should fail validation");
    }
}
