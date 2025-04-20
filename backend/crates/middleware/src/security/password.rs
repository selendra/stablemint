// backend/crates/middleware/src/security/password.rs
use app_config::AppConfig;
use app_error::{AppError, AppResult};
use argon2::{
    Argon2, Params, Algorithm, Version,
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString, rand_core::OsRng},
};
use tracing::{debug, error};

/// Hash a password using Argon2 with config settings
pub fn hash_password(password: &str) -> AppResult<String> {
    let salt = SaltString::generate(&mut OsRng);
    
    // Load configuration (this handles errors and returns defaults if loading fails)
    let config = AppConfig::load().unwrap_or_default();
    let argon2_config = &config.security.password.argon2;
    
    // Parse the variant from configuration
    let algorithm = match argon2_config.variant.to_lowercase().as_str() {
        "argon2i" => Algorithm::Argon2i,
        "argon2d" => Algorithm::Argon2d,
        _ => Algorithm::Argon2id, // Default to Argon2id for any other value
    };
    
    // Create custom parameters from config
    let params = Params::new(
        argon2_config.memory,              // Memory cost (in KiB)
        argon2_config.iterations,          // Number of iterations
        argon2_config.parallelism,         // Degree of parallelism
        Some(64),                          // Output length (fixed for compatibility)
    )
    .map_err(|e| {
        error!("Failed to create Argon2 params: {}", e);
        AppError::ServerError(anyhow::anyhow!("Failed to create Argon2 params: {}", e))
    })?;
    
    // Create Argon2 instance with the configured parameters
    let argon2 = Argon2::new(algorithm, Version::V0x13, params);

    debug!("Hashing password with Argon2 variant: {}", argon2_config.variant);
    let password_hash = argon2
        .hash_password(password.as_bytes(), &salt)
        .map_err(|e| {
            error!("Failed to hash password: {}", e);
            AppError::ServerError(anyhow::anyhow!("Failed to hash password: {}", e))
        })?
        .to_string();

    Ok(password_hash)
}

/// Verify a password against a stored hash
pub fn verify_password(password: &str, password_hash: &str) -> AppResult<bool> {
    let parsed_hash = PasswordHash::new(password_hash).map_err(|e| {
        error!("Invalid password hash: {}", e);
        AppError::ServerError(anyhow::anyhow!("Invalid password hash: {}", e))
    })?;

    // For verification, we need to use the same algorithm that was used for hashing
    // The hash string already contains the parameters, so we can just use the default Argon2
    let is_valid = Argon2::default()
        .verify_password(password.as_bytes(), &parsed_hash)
        .is_ok();

    debug!("Password verification result: {}", is_valid);
    Ok(is_valid)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_password_hash_and_verify() {
        let password = "secure_password123";

        let hash = hash_password(password).expect("Should hash password");

        let verified = verify_password(password, &hash).expect("Should verify password");
        assert!(verified, "Password verification should succeed");

        let wrong_password = "wrong_password";
        let verified_wrong =
            verify_password(wrong_password, &hash).expect("Should verify password");
        assert!(!verified_wrong, "Wrong password verification should fail");
    }
}