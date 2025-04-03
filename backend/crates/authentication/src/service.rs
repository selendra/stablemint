use app_database::service::DbService;
use app_error::{AppError, AppResult};
use app_models::user::{AuthResponse, LoginInput, RegisterInput, User, UserProfile};
use async_trait::async_trait;
use std::sync::Arc;
use tracing::{error, info};

use crate::{password, rate_limiter::LoginRateLimiter, validation, JwtService};

/// Trait defining the authentication service interface
#[async_trait]
pub trait AuthServiceTrait: Send + Sync {
    /// Register a new user
    async fn register(&self, input: RegisterInput) -> AppResult<AuthResponse>;

    /// Login an existing user
    async fn login(&self, input: LoginInput) -> AppResult<AuthResponse>;

    /// Get a user by their ID
    async fn get_user_by_id(&self, user_id: &str) -> AppResult<UserProfile>;

    /// Get the JWT service
    fn get_jwt_service(&self) -> Arc<JwtService>;
}

/// Implementation of the authentication service
pub struct AuthService {
    jwt_service: Arc<JwtService>,
    user_db: Option<Arc<DbService<'static, User>>>,
    rate_limiter: Option<Arc<LoginRateLimiter>>,
}

impl AuthService {
    /// Create a new authentication service with the given JWT secret
    pub fn new(jwt_secret: &[u8], expiry_hours: u64) -> Self {
        Self {
            jwt_service: Arc::new(JwtService::new(jwt_secret, expiry_hours)),
            user_db: None,
            rate_limiter: None,
        }
    }

    /// Add a database service to the authentication service
    pub fn with_db(mut self, user_db: Arc<DbService<'static, User>>) -> Self {
        self.user_db = Some(user_db);
        self
    }

     // Add rate limiter
     pub fn with_rate_limiter(mut self, rate_limiter: Arc<LoginRateLimiter>) -> Self {
        self.rate_limiter = Some(rate_limiter);
        self
    }

}

#[async_trait]
impl AuthServiceTrait for AuthService {
    fn get_jwt_service(&self) -> Arc<JwtService> {
        Arc::clone(&self.jwt_service)
    }

    async fn register(&self, input: RegisterInput) -> AppResult<AuthResponse> {
        // Sanitize and validate all inputs
        let name = validation::sanitize_string(&input.name);
        let username = validation::sanitize_string(&input.username);
        let email = validation::sanitize_string(&input.email);
        let password = input.password.clone(); // Don't trim password as it could contain meaningful spaces

        // Validate each field
        validation::validate_name(&name)?;
        validation::validate_username(&username)?;
        validation::validate_email(&email)?;
        validation::validate_password(&password)?;

        // Check if user already exists
        if let Some(user_db) = &self.user_db {
            let existing_users = user_db
                .get_records_by_field("username", username.clone())
                .await
                .map_err(|e| {
                    error!("Database error when checking for existing user: {}", e);
                    AppError::DatabaseError(anyhow::anyhow!(e))
                })?;

            if !existing_users.is_empty() {
                return Err(AppError::ValidationError(
                    "Username already taken".to_string(),
                ));
            }

            let existing_emails = user_db
                .get_records_by_field("email", email.clone())
                .await
                .map_err(|e| {
                    error!("Database error when checking for existing email: {}", e);
                    AppError::DatabaseError(anyhow::anyhow!(e))
                })?;

            if !existing_emails.is_empty() {
                return Err(AppError::ValidationError(
                    "Email already registered".to_string(),
                ));
            }
        }

        // Hash password
        let hashed_password = password::hash_password(&password)?;

        // Generate wallet info (in a real app this would use a crypto library)
        let address = format!("0x{}", hex::encode(uuid::Uuid::new_v4().as_bytes()));
        let private_key = format!("0x{}", hex::encode(uuid::Uuid::new_v4().as_bytes()));

        // Create new user with sanitized inputs
        let user = User::new(
            name,
            username.clone(),
            email,
            hashed_password,
            address,
            private_key,
        );

        // Rest of the method remains the same...
        // Store user if database is available
        let stored_user = if let Some(user_db) = &self.user_db {
            info!("Storing new user in database: {}", user.username);

            match user_db.create_record(user.clone()).await {
                Ok(Some(stored)) => stored,
                Ok(None) => {
                    error!("Database did not return stored user");
                    user.clone() // Use the original user as fallback
                }
                Err(e) => {
                    error!("Failed to store user in database: {}", e);
                    return Err(AppError::DatabaseError(anyhow::anyhow!(e)));
                }
            }
        } else {
            error!("Database not available for storing user");
            user.clone()
        };

        // Generate JWT token
        let token = self
            .jwt_service
            .generate_token(&stored_user.id.id.to_string(), &stored_user.username)?;

        // Create user profile
        let profile = UserProfile::from(stored_user);

        Ok(AuthResponse {
            token,
            user: profile,
        })
    }

    async fn login(&self, input: LoginInput) -> AppResult<AuthResponse> {
        // Sanitize inputs
        let username = validation::sanitize_string(&input.username);
        let password = input.password.clone(); // Don't trim password

        // Basic validation
        if username.is_empty() {
            return Err(AppError::ValidationError("Username cannot be empty".to_string()));
        }

        if password.is_empty() {
            return Err(AppError::ValidationError("Password cannot be empty".to_string()));
        }

        // Check rate limiting if enabled
        if let Some(rate_limiter) = &self.rate_limiter {
            // Use IP address or username as identifier (preferably IP in real implementation)
            rate_limiter.check_rate_limit(&username).await?;
        }

        if let Some(user_db) = &self.user_db {
            // Find user by username
            let users = user_db
                .get_records_by_field("username", username.clone())
                .await
                .map_err(|e| {
                    error!("Database error when fetching user for login: {}", e);
                    AppError::DatabaseError(anyhow::anyhow!(e))
                })?;

            if users.is_empty() {
                // Record failed attempt if rate limiting is enabled
                if let Some(rate_limiter) = &self.rate_limiter {
                    rate_limiter.record_failed_attempt(&username).await;
                }
                
                return Err(AppError::AuthenticationError(
                    "Invalid username or password".to_string(),
                ));
            }

            let user = &users[0];

            // Verify password
            let is_valid = password::verify_password(&password, &user.password)?;
            if !is_valid {
                // Record failed attempt if rate limiting is enabled
                if let Some(rate_limiter) = &self.rate_limiter {
                    rate_limiter.record_failed_attempt(&username).await;
                }
                
                // For security, use the same error message as when username is not found
                return Err(AppError::AuthenticationError(
                    "Invalid username or password".to_string(),
                ));
            }

            // Record successful attempt if rate limiting is enabled
            if let Some(rate_limiter) = &self.rate_limiter {
                rate_limiter.record_successful_attempt(&username).await;
            }

            // Generate JWT token
            let token = self
                .jwt_service
                .generate_token(&user.id.id.to_string(), &user.username)?;

            // Create user profile
            let profile = UserProfile::from(user.clone());

            Ok(AuthResponse {
                token,
                user: profile,
            })
        } else {
            Err(AppError::ServerError(anyhow::anyhow!(
                "Database not available"
            )))
        }
    }

    async fn get_user_by_id(&self, user_id: &str) -> AppResult<UserProfile> {
        if let Some(user_db) = &self.user_db {
            let clean_id = user_id.trim_start_matches('⟨').trim_end_matches('⟩');

            let user = user_db
                .get_record_by_id(clean_id)
                .await
                .map_err(|e| {
                    error!("Database error when fetching user by ID: {}", e);
                    AppError::DatabaseError(anyhow::anyhow!(e))
                })?
                .ok_or_else(|| AppError::NotFoundError("User not found".to_string()))?;

            Ok(UserProfile::from(user))
        } else {
            Err(AppError::ServerError(anyhow::anyhow!(
                "Database not available"
            )))
        }
    }
}

// For testing purposes
#[cfg(test)]
pub mod mocks {
    use super::*;
    use app_error::{AppError, AppResult};
    use app_models::user::{AuthResponse, LoginInput, RegisterInput, UserProfile};
    use async_trait::async_trait;
    use std::sync::{Arc, Mutex};

    pub struct MockAuthService {
        jwt_service: Arc<JwtService>,
        users: Arc<Mutex<Vec<User>>>,
    }

    impl MockAuthService {
        pub fn new(jwt_secret: &[u8]) -> Self {
            Self {
                jwt_service: Arc::new(JwtService::new(jwt_secret, 10)),
                users: Arc::new(Mutex::new(Vec::new())),
            }
        }
    }

    #[async_trait]
    impl AuthServiceTrait for MockAuthService {
        fn get_jwt_service(&self) -> Arc<JwtService> {
            Arc::clone(&self.jwt_service)
        }

        async fn register(&self, input: RegisterInput) -> AppResult<AuthResponse> {
            // Create a new user
            let user = User::new(
                input.name,
                input.username.clone(),
                input.email,
                input.password, // In mock, we don't hash the password
                "0xmockaddress".to_string(),
                "0xmockprivatekey".to_string(),
            );

            let profile = UserProfile::from(user.clone());
            let token = self
                .jwt_service
                .generate_token(&user.id.id.to_string(), &user.username)?;

            // Store the user
            self.users.lock().unwrap().push(user);

            Ok(AuthResponse {
                token,
                user: profile,
            })
        }

        async fn login(&self, input: LoginInput) -> AppResult<AuthResponse> {
            // Find the user
            let users = self.users.lock().unwrap();
            let user = users
                .iter()
                .find(|u| u.username == input.username)
                .ok_or_else(|| {
                    AppError::AuthenticationError("Invalid username or password".to_string())
                })?;

            // In mock, we don't verify the password, we just check equality
            if user.password != input.password {
                return Err(AppError::AuthenticationError(
                    "Invalid username or password".to_string(),
                ));
            }

            let profile = UserProfile::from(user.clone());
            let token = self
                .jwt_service
                .generate_token(&user.id.id.to_string(), &user.username)?;

            Ok(AuthResponse {
                token,
                user: profile,
            })
        }

        async fn get_user_by_id(&self, user_id: &str) -> AppResult<UserProfile> {
            let users = self.users.lock().unwrap();
            let user = users
                .iter()
                .find(|u| u.id.id.to_string() == user_id)
                .ok_or_else(|| AppError::NotFoundError("User not found".to_string()))?;

            Ok(UserProfile::from(user.clone()))
        }
    }
}
