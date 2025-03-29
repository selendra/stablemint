use crate::errors::{AppError, AppResult};
use chrono::{Duration, Utc};
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation, decode, encode};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,      // Subject (user ID)
    pub exp: i64,         // Expiration time
    pub iat: i64,         // Issued at
    pub username: String, // Username for convenience
}

pub struct JwtService {
    encoding_key: EncodingKey,
    decoding_key: DecodingKey,
}

impl JwtService {
    pub fn new(secret: &[u8]) -> Self {
        Self {
            encoding_key: EncodingKey::from_secret(secret),
            decoding_key: DecodingKey::from_secret(secret),
        }
    }

    pub fn generate_token(&self, user_id: &str, username: &str) -> AppResult<String> {
        let now = Utc::now();
        let expires_at = now + Duration::hours(24);

        let claims = Claims {
            sub: user_id.to_string(),
            iat: now.timestamp(),
            exp: expires_at.timestamp(),
            username: username.to_string(),
        };

        encode(&Header::default(), &claims, &self.encoding_key)
            .map_err(|e| AppError::AuthenticationError(format!("Failed to generate token: {}", e)))
    }

    pub fn validate_token(&self, token: &str) -> AppResult<Claims> {
        let token_data = decode::<Claims>(token, &self.decoding_key, &Validation::default())
            .map_err(|e| AppError::AuthenticationError(format!("Invalid token: {}", e)))?;

        Ok(token_data.claims)
    }
}
