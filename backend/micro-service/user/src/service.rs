use app_database::service::DbService;
use app_error::{AppError, AppResult};
use app_middleware::{JwtService, RedisLoginRateLimiter, security::password, validation};
use app_models::{user::{AuthResponse, LoginInput, RegisterInput, User, UserProfile}, wallet::Wallet};
use app_utils::generate::EthereumWallet;
use async_trait::async_trait;
use std::sync::Arc;
use tracing::{error, info};

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

/// Validation container to reduce boilerplate
#[derive(Debug)]
struct ValidationInput {
    name: String,
    username: String,
    email: String,
    password: String,
}

impl ValidationInput {
    fn from_register_input(input: RegisterInput) -> Self {
        Self {
            name: validation::sanitize_string(&input.name),
            username: validation::sanitize_string(&input.username),
            email: validation::sanitize_string(&input.email),
            password: input.password,
        }
    }

    fn from_login_input(input: LoginInput) -> Self {
        Self {
            name: String::new(), // Not used for login
            username: validation::sanitize_string(&input.username),
            email: String::new(), // Not used for login
            password: input.password,
        }
    }

    // Validate all fields for registration
    fn validate_registration(&self) -> AppResult<()> {
        validation::validate_name(&self.name)?;
        validation::validate_username(&self.username)?;
        validation::validate_email(&self.email)?;
        validation::validate_password(&self.password)?;
        Ok(())
    }

    // Validate for login (only username and password)
    fn validate_login(&self) -> AppResult<()> {
        if self.username.is_empty() {
            return Err(AppError::ValidationError(
                "Username cannot be empty".to_string(),
            ));
        }

        if self.password.is_empty() {
            return Err(AppError::ValidationError(
                "Password cannot be empty".to_string(),
            ));
        }

        Ok(())
    }
}

/// Implementation of the authentication service
pub struct AuthService {
    jwt_service: Arc<JwtService>,
    rate_limiter: Option<Arc<RedisLoginRateLimiter>>, // Changed to Redis implementation
    user_db: Option<Arc<DbService<'static, User>>>,
    wallet_db: Option<Arc<DbService<'static, Wallet>>>, 
}

impl AuthService {
    /// Create a new authentication service with the given JWT secret
    pub fn new(jwt_secret: &[u8], expiry_hours: u64) -> Self {
        Self {
            jwt_service: Arc::new(JwtService::new(jwt_secret, expiry_hours)),
            rate_limiter: None,
            user_db: None,
            wallet_db: None,
        }
    }

    pub fn with_wallet_db(mut self, wallet_db: Arc<DbService<'static, Wallet>>) -> Self {
        self.wallet_db = Some(wallet_db);
        self
    }

    /// Add a database service to the authentication service
    pub fn with_db(mut self, user_db: Arc<DbService<'static, User>>) -> Self {
        self.user_db = Some(user_db);
        self
    }

    /// Add rate limiter to the authentication service
    pub fn with_rate_limiter(mut self, rate_limiter: Arc<RedisLoginRateLimiter>) -> Self {
        self.rate_limiter = Some(rate_limiter);
        self
    }

    // Helper method to check if a user with the given username or email exists
    async fn check_user_exists<'a>(&self, username: &'a str, email: &'a str) -> AppResult<()> {
        if let Some(user_db) = &self.user_db {
            // Check username
            let existing_users = user_db
                .get_records_by_field("username", username.to_string())
                .await
                .map_err(|e| {
                    error!("Database error when checking for existing user: {}", e);
                    AppError::DatabaseError(anyhow::anyhow!(e))
                })?;

            if !existing_users.is_empty() {
                return Err(AppError::ResourceExistsError(
                    "This username is already registered. Please choose a different username."
                        .to_string(),
                ));
            }

            // Check email if provided
            if !email.is_empty() {
                let existing_emails = user_db
                    .get_records_by_field("email", email.to_string())
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
        } else {
            return Err(AppError::ServerError(anyhow::anyhow!(
                "Database not available"
            )));
        }

        Ok(())
    }

    // Helper method to get user by username
    async fn get_user_by_username(&self, username: &str) -> AppResult<User> {
        if let Some(user_db) = &self.user_db {
            let users = user_db
                .get_records_by_field("username", username.to_string())
                .await
                .map_err(|e| {
                    error!("Database error when fetching user for login: {}", e);
                    AppError::DatabaseError(anyhow::anyhow!(e))
                })?;

            if users.is_empty() {
                return Err(AppError::AuthenticationError(
                    "Login failed: The username or password you entered is incorrect".to_string(),
                ));
            }

            Ok(users[0].clone())
        } else {
            Err(AppError::ServerError(anyhow::anyhow!(
                "Database not available"
            )))
        }
    }

    // Helper method to create authentication response
    fn create_auth_response(&self, user: &User) -> AppResult<AuthResponse> {
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
    }

    // Helper to format user ID correctly
    fn clean_user_id(user_id: &str) -> String {
        user_id
            .trim_start_matches('⟨')
            .trim_end_matches('⟩')
            .to_string()
    }
}

#[async_trait]
impl AuthServiceTrait for AuthService {
    fn get_jwt_service(&self) -> Arc<JwtService> {
        Arc::clone(&self.jwt_service)
    }

    async fn register(&self, input: RegisterInput) -> AppResult<AuthResponse> {
        // Add a with_wallet_db method
      
        // Extract and validate input
        let input = ValidationInput::from_register_input(input);
        input.validate_registration()?;

        // Check if user already exists
        self.check_user_exists(&input.username, &input.email)
            .await?;

        // Hash password
        let hashed_password = password::hash_password(&input.password)?;

        // Generate wallet using the wallet microservice
        let ethereum_wallet = EthereumWallet::new();
        let address = ethereum_wallet.address().to_string();
        let private_key = ethereum_wallet.private_key_hex();
        let mnemonic = ethereum_wallet.mnemonic_phrase();


        // Create new user with sanitized inputs
        let user = User::new(
            input.name,
            input.username,
            input.email,
            hashed_password,
            address.clone(),
        );

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

         // Store wallet in separate database if available
        if let Some(wallet_db) = &self.wallet_db {
            info!("Creating wallet for user: {}", stored_user.username);
            
            // Create wallet record
            let wallet = Wallet {
                id: Wallet::generate_id(),
                user_email: stored_user.email.clone(),
                address,
                private_key,
                mnemonic,
                created_at: chrono::Utc::now(),
                updated_at: chrono::Utc::now(),
            };
            
            // Store wallet in database
            match wallet_db.create_record(wallet).await {
                Ok(Some(_)) => {
                    info!("Wallet created successfully for user: {}", stored_user.username);
                }
                Ok(None) => {
                    error!("Database did not return stored wallet for user: {}", stored_user.username);
                    // Consider how to handle this case - perhaps retry or alert
                }
                Err(e) => {
                    error!("Failed to store wallet in database");
                    return Err(AppError::DatabaseError(anyhow::anyhow!(e)));
                }
            }
        }
        // Create authentication response
        self.create_auth_response(&stored_user)
    }

    async fn login(&self, input: LoginInput) -> AppResult<AuthResponse> {
        // Extract and validate input
        let input = ValidationInput::from_login_input(input);
        input.validate_login()?;

        // Check rate limiting if enabled
        if let Some(rate_limiter) = &self.rate_limiter {
            // Check if the account is rate limited
            rate_limiter.check_rate_limit(&input.username).await?;
        }

        // Get user by username
        let user = match self.get_user_by_username(&input.username).await {
            Ok(user) => user,
            Err(e) => {
                // Record failed attempt if rate limiting is enabled
                if let Some(rate_limiter) = &self.rate_limiter {
                    if let Err(e) = rate_limiter.record_failed_attempt(&input.username).await {
                        error!("Failed to record rate limit attempt: {}", e);
                        // Optionally, you could decide whether to proceed or return the error
                    }
                }

                return Err(e);
            }
        };

        // Verify password
        let is_valid = password::verify_password(&input.password, &user.password)?;
        if !is_valid {
            // Record failed attempt if rate limiting is enabled
            if let Some(rate_limiter) = &self.rate_limiter {
                if let Err(e) = rate_limiter.record_failed_attempt(&input.username).await {
                    error!("Failed to record rate limit attempt: {}", e);
                    // Optionally, you could decide whether to proceed or return the error
                }
            }

            // For security, use the same error message as when username is not found
            return Err(AppError::AuthenticationError(
                "Login failed: The username or password you entered is incorrect".to_string(),
            ));
        }

        // Record successful attempt if rate limiting is enabled
        if let Some(rate_limiter) = &self.rate_limiter {
            if let Err(e) = rate_limiter.record_failed_attempt(&input.username).await {
                error!("Failed to record rate limit attempt: {}", e);
                // Optionally, you could decide whether to proceed or return the error
            }
        }

        // Create authentication response
        self.create_auth_response(&user)
    }

    async fn get_user_by_id(&self, user_id: &str) -> AppResult<UserProfile> {
        if let Some(user_db) = &self.user_db {
            let clean_id = Self::clean_user_id(user_id);

            let user = user_db
                .get_record_by_id(&clean_id)
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
                "0xmockaddress".to_string()
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
                    AppError::AuthenticationError(
                        "Login failed: The username or password you entered is incorrect"
                            .to_string(),
                    )
                })?;

            // In mock, we don't verify the password, we just check equality
            if user.password != input.password {
                return Err(AppError::AuthenticationError(
                    "Login failed: The username or password you entered is incorrect".to_string(),
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
