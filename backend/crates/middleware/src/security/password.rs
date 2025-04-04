use app_error::{AppError, AppResult};
use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString}, Argon2
};
use tracing::{debug, error};

/// Hash a password using Argon2id
pub fn hash_password(password: &str) -> AppResult<String> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();

    debug!("Hashing password");
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
