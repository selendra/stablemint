use crate::{
    database::operation::DbService,
    errors::{AppError, AppResult},
    middleware::auth::{
        jwt::JwtService,
        password::{self},
    },
    models::user::{AuthResponse, LoginInput, RegisterInput, User, UserProfile},
};
use std::sync::Arc;
use tracing::{error, info};

pub struct AuthService {
    jwt_service: Arc<JwtService>,
    user_db: Option<Arc<DbService<'static, User>>>,
}

impl AuthService {
    pub fn new(jwt_secret: &[u8]) -> Self {
        Self {
            jwt_service: Arc::new(JwtService::new(jwt_secret)),
            user_db: None,
        }
    }

    // Add database service
    pub fn with_db(mut self, user_db: Arc<DbService<'static, User>>) -> Self {
        self.user_db = Some(user_db);
        self
    }

    pub fn get_jwt_service(&self) -> Arc<JwtService> {
        Arc::clone(&self.jwt_service)
    }

    pub async fn register(&self, input: RegisterInput) -> AppResult<AuthResponse> {
        // Validate input (add more validation as needed)
        if input.username.trim().is_empty() || input.password.trim().is_empty() {
            return Err(AppError::ValidationError(
                "Username and password are required".to_string(),
            ));
        }

        // Check if user already exists
        if let Some(user_db) = &self.user_db {
            let existing_users = user_db
                .get_records_by_field("username", input.username.clone())
                .await
                .map_err(|e| {
                    error!("Database error when checking for existing user: {}", e);
                    AppError::DatabaseError(e)
                })?;

            if !existing_users.is_empty() {
                return Err(AppError::ValidationError(
                    "Username already taken".to_string(),
                ));
            }

            let existing_emails = user_db
                .get_records_by_field("email", input.email.clone())
                .await
                .map_err(|e| {
                    error!("Database error when checking for existing email: {}", e);
                    AppError::DatabaseError(e)
                })?;

            if !existing_emails.is_empty() {
                return Err(AppError::ValidationError(
                    "Email already registered".to_string(),
                ));
            }
        }

        // Hash password
        let hashed_password = password::hash_password(&input.password)?;

        // Generate wallet info (in a real app this would use a crypto library)
        let address = format!("0x{}", hex::encode(uuid::Uuid::new_v4().as_bytes()));
        let private_key = format!("0x{}", hex::encode(uuid::Uuid::new_v4().as_bytes()));

        // Create new user
        let user = User::new(
            input.name,
            input.username.clone(),
            input.email,
            hashed_password,
            address,
            private_key,
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
                    return Err(AppError::DatabaseError(e));
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

    pub async fn login(&self, input: LoginInput) -> AppResult<AuthResponse> {
        if let Some(user_db) = &self.user_db {
            // Find user by username
            let users = user_db
                .get_records_by_field("username", input.username.clone())
                .await
                .map_err(|e| {
                    error!("Database error when fetching user for login: {}", e);
                    AppError::DatabaseError(e)
                })?;

            if users.is_empty() {
                return Err(AppError::AuthenticationError(
                    "Invalid username or password".to_string(),
                ));
            }

            let user = &users[0];

            // Verify password
            let is_valid = password::verify_password(&input.password, &user.password)?;
            if !is_valid {
                return Err(AppError::AuthenticationError(
                    "Invalid username or password".to_string(),
                ));
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

    // Get user by ID
    pub async fn get_user_by_id(&self, user_id: &str) -> AppResult<UserProfile> {
        if let Some(user_db) = &self.user_db {
            let clean_id = user_id.trim_start_matches('⟨').trim_end_matches('⟩');

            let user = user_db
                .get_record_by_id(clean_id)
                .await
                .map_err(|e| {
                    error!("Database error when fetching user by ID: {}", e);
                    AppError::DatabaseError(e)
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
