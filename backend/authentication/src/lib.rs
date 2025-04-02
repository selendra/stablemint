use anyhow::Result;
use axum::{
    extract::{FromRef, FromRequestParts},
    http::{request::Parts, StatusCode},
};
use chrono::{ Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use stablemint_error::AppError;
use std::env;

// JWT Claims structure
#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,        // Subject (user ID)
    pub exp: usize,         // Expiration time (as UTC timestamp)
    pub iat: usize,         // Issued at (as UTC timestamp)
    pub role: String,       // User role
    pub address: String,    // User wallet address
}

// JWT configuration
#[derive(Clone, Debug)]
pub struct JwtConfig {
    pub secret: String,
    pub expiration: Duration, // Token expiration time
}

impl JwtConfig {
    pub fn from_env() -> Result<Self, AppError> {
        let secret = env::var("JWT_SECRET").map_err(|_| {
            AppError::ConfigError("JWT_SECRET environment variable not set".to_string())
        })?;

        // Default expiration: 1 day
        let expiration_hours = env::var("JWT_EXPIRATION_HOURS")
            .unwrap_or_else(|_| "24".to_string())
            .parse::<i64>()
            .map_err(|_| {
                AppError::ConfigError("Invalid JWT_EXPIRATION_HOURS value".to_string())
            })?;

        Ok(Self {
            secret,
            expiration: Duration::hours(expiration_hours),
        })
    }
}

#[derive(Clone)]
pub struct JwtAuth {
    pub config: JwtConfig,
}

impl JwtAuth {
    pub fn new(config: JwtConfig) -> Self {
        Self { config }
    }

    // Generate a JWT token for a user
    pub fn generate_token(&self, user_id: &str, role: &str, address: &str) -> Result<String, AppError> {
        let now = Utc::now();
        let expiration = now + self.config.expiration;
        
        let claims = Claims {
            sub: user_id.to_string(),
            exp: expiration.timestamp() as usize,
            iat: now.timestamp() as usize,
            role: role.to_string(),
            address: address.to_string(),
        };

        encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(self.config.secret.as_bytes()),
        )
        .map_err(|e| AppError::Internal(anyhow::anyhow!("Token generation failed: {}", e)))
    }

    // Validate a JWT token
    pub fn validate_token(&self, token: &str) -> Result<Claims, AppError> {
        decode::<Claims>(
            token,
            &DecodingKey::from_secret(self.config.secret.as_bytes()),
            &Validation::default(),
        )
        .map(|data| data.claims)
        .map_err(|e| {
            match e.kind() {
                jsonwebtoken::errors::ErrorKind::ExpiredSignature => {
                    AppError::AuthError("Token expired".to_string())
                }
                _ => AppError::AuthError(format!("Invalid token: {}", e)),
            }
        })
    }

    // Extract token from Authorization header
    pub fn extract_token_from_header(auth_header: &str) -> Result<&str, AppError> {
        if !auth_header.starts_with("Bearer ") {
            return Err(AppError::AuthError(
                "Authorization header must start with 'Bearer '".to_string(),
            ));
        }

        Ok(&auth_header[7..]) // Remove "Bearer " prefix
    }
}

// AuthUser represents the authenticated user extracted from JWT
#[derive(Debug, Clone)]
pub struct AuthUser {
    pub id: String,
    pub role: String,
    pub address: String,
}


impl<S> FromRequestParts<S> for AuthUser
where
    JwtAuth: FromRef<S>,
    S: Send + Sync,
{
    type Rejection = (StatusCode, &'static str);

    fn from_request_parts(
        parts: &mut Parts,
        state: &S,
    ) -> impl Future<Output = Result<Self, Self::Rejection>> + Send {
        async move {
            // Extract JWT auth from app state
            let jwt_auth = JwtAuth::from_ref(state);

            // Extract authorization header
            let auth_header = parts
                .headers
                .get("Authorization")
                .ok_or((StatusCode::UNAUTHORIZED, "Missing Authorization header"))?
                .to_str()
                .map_err(|_| (StatusCode::BAD_REQUEST, "Invalid Authorization header"))?;

            // Extract and validate token
            let token = JwtAuth::extract_token_from_header(auth_header)
                .map_err(|_| (StatusCode::UNAUTHORIZED, "Invalid Authorization format"))?;

            let claims = jwt_auth
                .validate_token(token)
                .map_err(|_| (StatusCode::UNAUTHORIZED, "Invalid token"))?;

            Ok(AuthUser {
                id: claims.sub,
                role: claims.role,
                address: claims.address,
            })
        }
    }
}

// Role-based authorization middleware
pub fn authorize(required_role: &'static str) -> impl Fn(AuthUser) -> Result<AuthUser, (StatusCode, &'static str)> {
    move |user: AuthUser| {
        if user.role == required_role || user.role == "Admin" {
            Ok(user)
        } else {
            Err((
                StatusCode::FORBIDDEN,
                "Insufficient permissions for this operation",
            ))
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::{HeaderMap, HeaderValue, Request};
    use axum::body::Body;
    use axum::extract::FromRef;
    use chrono::Utc;
    
    // Mock implementation for testing
    impl FromRef<AppState> for JwtAuth {
        fn from_ref(state: &AppState) -> Self {
            state.jwt_auth.clone()
        }
    }

    // Mock app state
    #[derive(Clone)]
    struct AppState {
        jwt_auth: JwtAuth,
    }

    #[test]
    fn test_jwt_config_from_env() {
        // Setup test environment variables
        unsafe { std::env::set_var("JWT_SECRET", "test_secret_key") };
        unsafe { std::env::set_var("JWT_EXPIRATION_HOURS", "48") };
        
        // Test config creation
        let config = JwtConfig::from_env().unwrap();
        assert_eq!(config.secret, "test_secret_key");
        assert_eq!(config.expiration, Duration::hours(48));
        
        // Test with default expiration
        unsafe { std::env::remove_var("JWT_EXPIRATION_HOURS") };
        let config = JwtConfig::from_env().unwrap();
        assert_eq!(config.expiration, Duration::hours(24));
    }

    #[test]
    fn test_generate_token() {
        let config = JwtConfig {
            secret: "test_secret".to_string(),
            expiration: Duration::hours(24),
        };
        
        let jwt_auth = JwtAuth::new(config);
        let token = jwt_auth.generate_token("user123", "User", "0x123").unwrap();
        
        // Verify token is not empty
        assert!(!token.is_empty());
    }

    #[test]
    fn test_validate_token() {
        let config = JwtConfig {
            secret: "test_secret".to_string(),
            expiration: Duration::hours(24),
        };
        
        let jwt_auth = JwtAuth::new(config);
        let token = jwt_auth.generate_token("user123", "User", "0x123").unwrap();
        
        // Validate the token
        let claims = jwt_auth.validate_token(&token).unwrap();
        
        // Verify claims content
        assert_eq!(claims.sub, "user123");
        assert_eq!(claims.role, "User");
        assert_eq!(claims.address, "0x123");
    }

    #[test]
    fn test_extract_token_from_header() {
        // Valid header
        let auth_header = "Bearer token123";
        let token = JwtAuth::extract_token_from_header(auth_header).unwrap();
        assert_eq!(token, "token123");
        
        // Invalid header format
        let auth_header = "Basic token123";
        let result = JwtAuth::extract_token_from_header(auth_header);
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_auth_user_extraction() {
        // Create JWT config and auth
        let config = JwtConfig {
            secret: "test_secret".to_string(),
            expiration: Duration::hours(24),
        };
        let jwt_auth = JwtAuth::new(config.clone());
        
        // Generate token
        let token = jwt_auth.generate_token("user123", "User", "0x123").unwrap();
        
        // Create app state
        let app_state = AppState { jwt_auth };
        
        // Create HTTP request parts with auth header
        let mut headers = HeaderMap::new();
        headers.insert(
            "Authorization", 
            HeaderValue::from_str(&format!("Bearer {}", token)).unwrap()
        );
        
        let req = Request::builder()
            .header("Authorization", format!("Bearer {}", token))
            .body(Body::empty())
            .unwrap();
            
        let (mut parts, _) = req.into_parts();
        
        // Extract auth user
        let auth_user = AuthUser::from_request_parts(&mut parts, &app_state).await.unwrap();
        
        // Verify extracted user data
        assert_eq!(auth_user.id, "user123");
        assert_eq!(auth_user.role, "User");
        assert_eq!(auth_user.address, "0x123");
    }

    #[test]
    fn test_authorize_middleware() {
        // Create users with different roles
        let admin_user = AuthUser {
            id: "admin123".to_string(),
            role: "Admin".to_string(),
            address: "0xadmin".to_string(),
        };
        
        let user = AuthUser {
            id: "user123".to_string(),
            role: "User".to_string(),
            address: "0xuser".to_string(),
        };
        
        let guest = AuthUser {
            id: "guest123".to_string(),
            role: "Guest".to_string(),
            address: "0xguest".to_string(),
        };
        
        // Test admin access (should always work)
        let admin_fn = authorize("User");
        assert!(admin_fn(admin_user.clone()).is_ok());
        
        // Test correct role
        let user_fn = authorize("User");
        assert!(user_fn(user.clone()).is_ok());
        
        // Test incorrect role
        let user_fn = authorize("User");
        assert!(user_fn(guest.clone()).is_err());
    }

     #[test]
    fn test_token_expiration() {
        // Create a JWT validation context with expiration validation enabled
        let mut validation = Validation::default();
        validation.validate_exp = true; // Ensure expiration validation is enabled
        validation.leeway = 0;         // No leeway to ensure strict time checking

        // Create configuration
        let config = JwtConfig {
            secret: "test_secret".to_string(),
            expiration: Duration::hours(24),
        };
        
        let jwt_auth = JwtAuth::new(config);
        
        // Create claims that are definitely expired
        // Setting expiration to 1 hour in the past
        let now = Utc::now();
        let exp_time = now - Duration::hours(1);
        
        let claims = Claims {
            sub: "user123".to_string(),
            exp: exp_time.timestamp() as usize,
            iat: (exp_time - Duration::minutes(5)).timestamp() as usize,
            role: "User".to_string(),
            address: "0x123".to_string(),
        };
        
        // Create token with expired claims
        let token = encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(jwt_auth.config.secret.as_bytes()),
        ).unwrap();
        
        // Attempt to validate (should fail with expired token)
        let result = jwt_auth.validate_token(&token);
        
        // Verify token validation fails
        assert!(result.is_err(), "Expected token validation to fail due to expiration");
        
        // Verify correct error type
        match result {
            Err(AppError::AuthError(msg)) => {
                assert!(msg.contains("Token expired"), "Error message should mention token expiration, got: {}", msg);
            },
            Err(e) => panic!("Expected AuthError with 'Token expired' message, got: {:?}", e),
            Ok(_) => panic!("Expected error but token validation succeeded"),
        }
    }

}