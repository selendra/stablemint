use crate::{
    error::{AppError, AppResult},
    middleware::auth::{
        jwt::JwtService,
        password::{hash_password, verify_password},
    },
    models::user::{AuthResponse, LoginInput, RegisterInput, User, UserProfile},
};
use chrono::Utc;
use std::sync::Arc;
use uuid::Uuid;

// Simulating a database for this example
// In a real app, this would use a database repository
pub struct AuthService {
    jwt_service: Arc<JwtService>,
    // In a real app: user_repository: Arc<dyn UserRepository>,
    // For this example, we'll use an in-memory Vec
    users: std::sync::Mutex<Vec<User>>,
}

impl AuthService {
    pub fn new(jwt_secret: &[u8]) -> Self {
        Self {
            jwt_service: Arc::new(JwtService::new(jwt_secret)),
            users: std::sync::Mutex::new(Vec::new()),
        }
    }

    pub async fn register(&self, input: RegisterInput) -> AppResult<AuthResponse> {
        // Check if user already exists
        {
            let users = self.users.lock().unwrap();
            if users.iter().any(|u| u.username == input.username) {
                return Err(AppError::ValidationError(
                    "Username already taken".to_string(),
                ));
            }
            if users.iter().any(|u| u.email == input.email) {
                return Err(AppError::ValidationError(
                    "Email already registered".to_string(),
                ));
            }
        }

        // Hash password
        let hashed_password = hash_password(&input.password)?;

        // Generate wallet info (in a real app this would use a crypto library)
        let address = format!("0x{}", hex::encode(Uuid::new_v4().as_bytes()));
        let private_key = format!("0x{}", hex::encode(Uuid::new_v4().as_bytes()));

        // Create new user
        let now = Utc::now();
        let user = User {
            id: Uuid::new_v4(),
            name: input.name,
            username: input.username.clone(),
            email: input.email,
            password: hashed_password,
            address,
            private_key,
            created_at: now,
            updated_at: now,
        };

        // Generate JWT token
        let token = self.jwt_service.generate_token(user.id, &user.username)?;

        // Create user profile
        let profile = UserProfile::from(user.clone());

        // Store user
        {
            let mut users = self.users.lock().unwrap();
            users.push(user);
        }

        Ok(AuthResponse {
            token,
            user: profile,
        })
    }

    pub async fn login(&self, input: LoginInput) -> AppResult<AuthResponse> {
        // Find user by username
        let user = {
            let users = self.users.lock().unwrap();
            users
                .iter()
                .find(|u| u.username == input.username)
                .cloned()
                .ok_or_else(|| {
                    AppError::AuthenticationError("Invalid username or password".to_string())
                })?
        };

        // Verify password
        let is_valid = verify_password(&input.password, &user.password)?;
        if !is_valid {
            return Err(AppError::AuthenticationError(
                "Invalid username or password".to_string(),
            ));
        }

        // Generate JWT token
        let token = self.jwt_service.generate_token(user.id, &user.username)?;

        // Create user profile
        let profile = UserProfile::from(user);

        Ok(AuthResponse {
            token,
            user: profile,
        })
    }

    // Additional method to get user by ID (useful for resolving JWT token to user)
    pub async fn get_user_by_id(&self, user_id: &str) -> AppResult<UserProfile> {
        let users = self.users.lock().unwrap();

        let uuid = Uuid::parse_str(user_id)
            .map_err(|_| AppError::ValidationError("Invalid user ID".to_string()))?;

        let user = users
            .iter()
            .find(|u| u.id == uuid)
            .cloned()
            .ok_or_else(|| AppError::NotFoundError("User not found".to_string()))?;

        Ok(UserProfile::from(user))
    }
}
