use app_error::{AppError, AppResult};
use chrono::{Duration, Utc};
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation, decode, encode};
use serde::{Deserialize, Serialize};
use tracing::{debug, error};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    pub sub: String,      // Subject (user ID)
    pub exp: i64,         // Expiration time
    pub iat: i64,         // Issued at
    pub username: String, // Username for convenience
}

pub struct JwtService {
    encoding_key: EncodingKey,
    decoding_key: DecodingKey,
    expiry_hours: u64,
}

impl JwtService {
    pub fn new(secret: &[u8], expiry_hours: u64) -> Self {
        Self {
            encoding_key: EncodingKey::from_secret(secret),
            decoding_key: DecodingKey::from_secret(secret),
            expiry_hours,
        }
    }

    pub fn generate_token(&self, user_id: &str, username: &str) -> AppResult<String> {
        let now = Utc::now();
        let expires_at = now + Duration::hours(self.expiry_hours as i64);

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
            .map_err(|e| {
                error!("Token validation failed: {}", e);
                AppError::AuthenticationError(format!("Invalid token: {}", e))
            })?;

        debug!("Token validated for user: {}", token_data.claims.username);
        Ok(token_data.claims)
    }
}

// Create a middleware to extract JWT from request headers
pub mod middleware {
    use crate::JwtService;
    use axum::{
        body::Body,
        extract::Request,
        http::{HeaderMap, header},
        middleware::Next,
        response::Response,
    };
    use std::sync::Arc;
    use tracing::{debug, warn};

    pub async fn jwt_auth(
        headers: HeaderMap,
        jwt_service: Arc<JwtService>,
        request: Request<Body>,
        next: Next,
    ) -> Response {
        if let Some(auth_header) = headers.get(header::AUTHORIZATION) {
            if let Ok(auth_str) = auth_header.to_str() {
                if auth_str.starts_with("Bearer ") {
                    let token = &auth_str["Bearer ".len()..];

                    match jwt_service.validate_token(token) {
                        Ok(claims) => {
                            debug!("JWT validated for user {}", claims.username);
                            // You could inject the claims into the request extensions here
                            // But we'll leave that for the specific implementation
                        }
                        Err(e) => {
                            warn!("JWT validation failed: {}", e);
                            // Continue without validated claims
                        }
                    }
                }
            }
        }

        next.run(request).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Helper function to create a test JWT service
    fn create_test_jwt_service() -> JwtService {
        let secret = b"test_secret_key_for_testing_purposes_only";
        JwtService::new(secret, 10)
    }

    #[test]
    fn test_jwt_token_generation() {
        let jwt_service = create_test_jwt_service();
        let user_id = "user123";
        let username = "testuser";

        let token = jwt_service.generate_token(user_id, username);
        assert!(token.is_ok(), "Token generation should succeed");

        let token_str = token.unwrap();
        assert!(!token_str.is_empty(), "Generated token should not be empty");
    }

    #[test]
    fn test_jwt_token_validation() {
        let jwt_service = create_test_jwt_service();
        let user_id = "user123";
        let username = "testuser";

        let token = jwt_service.generate_token(user_id, username).unwrap();
        let claims = jwt_service.validate_token(&token);

        assert!(
            claims.is_ok(),
            "Valid token should be validated successfully"
        );

        let validated_claims = claims.unwrap();
        assert_eq!(
            validated_claims.sub, user_id,
            "Subject claim should match user ID"
        );
        assert_eq!(
            validated_claims.username, username,
            "Username claim should match"
        );
    }

    #[test]
    fn test_jwt_token_validation_with_invalid_token() {
        let jwt_service = create_test_jwt_service();
        let invalid_token = "invalid.token.string";

        let result = jwt_service.validate_token(invalid_token);
        assert!(result.is_err(), "Invalid token should fail validation");
    }

    #[test]
    fn test_jwt_token_expiration() {
        let jwt_service = create_test_jwt_service();

        // Create claims with an already expired token
        let now = Utc::now();
        let expired_time = now - Duration::hours(1);

        let claims = Claims {
            sub: "user123".to_string(),
            iat: now.timestamp(),
            exp: expired_time.timestamp(), // Expired timestamp
            username: "testuser".to_string(),
        };

        let token = encode(&Header::default(), &claims, &jwt_service.encoding_key)
            .expect("Failed to encode token");

        let result = jwt_service.validate_token(&token);
        assert!(result.is_err(), "Expired token should fail validation");
    }
}
