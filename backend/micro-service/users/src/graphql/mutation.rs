use async_graphql::{Context, Error as GraphQLError, Object, Result as GraphQLResult};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use stablemint_authentication::{AuthUser, JwtAuth};
use stablemint_models::user::{CreateUserInput, DBUser, User, UserRole};
use stablemint_surrealdb::{services::DbService, types::Database};
use stablemint_utils::{hash_password, verify_password};
use std::sync::Arc;
use uuid::Uuid;

// Request types
#[derive(Deserialize, async_graphql::InputObject)]
struct LoginRequest {
    email: String,
    password: String,
}

// Response types
#[derive(Serialize, async_graphql::SimpleObject)]
struct LoginResponse {
    token: String,
    user: User,
}

// GraphQL Mutation Root
#[derive(Default)]
pub struct MutationRoot;

#[Object]
impl MutationRoot {
    async fn create_user<'ctx>(
        &self,
        ctx: &Context<'ctx>,
        input: CreateUserInput,
    ) -> GraphQLResult<User> {
        // For create_user, check if admin (for setting roles other than User)
        let auth_user = ctx.data::<AuthUser>().ok();

        // Only admins can create users with non-default roles
        if input.role != UserRole::User
            && (auth_user.is_none() || auth_user.unwrap().role != "Admin")
        {
            return Err(GraphQLError::new(
                "Not authorized to create users with this role",
            ));
        }

        let db = ctx
            .data::<Arc<Database>>()
            .map_err(|_| GraphQLError::new("Database connection error"))?;

        let user_service = DbService::<DBUser>::new(db, "users");

        // Check if email already exists
        let existing_users = user_service
            .get_records_by_field("email", input.email.clone())
            .await
            .map_err(|e| GraphQLError::new(format!("Database error: {}", e)))?;

        if !existing_users.is_empty() {
            return Err(GraphQLError::new("Email already in use"));
        }

        let hashed_password = hash_password(&input.password)?;

        // Create private key (in a real app, this would come from a crypto library)
        let private_key = format!("dummy_key_{}", Uuid::new_v4());

        let now = Utc::now();

        let new_user = DBUser {
            id: None,
            username: input.username,
            password: hashed_password,
            email: input.email,
            address: input.address,
            private_key,
            role: input.role,
            created_at: now,
            updated_at: now,
        };

        let created_user = user_service
            .create_record(new_user)
            .await
            .map_err(|e| GraphQLError::new(format!("Failed to create user: {}", e)))?
            .ok_or_else(|| GraphQLError::new("Failed to create user"))?;

        Ok(User::from_db(created_user))
    }

    async fn login<'ctx>(
        &self,
        ctx: &Context<'ctx>,
        input: LoginRequest,
    ) -> GraphQLResult<LoginResponse> {
        let db = ctx
            .data::<Arc<Database>>()
            .map_err(|_| GraphQLError::new("Database connection error"))?;

        let jwt_auth = ctx
            .data::<Arc<JwtAuth>>()
            .map_err(|_| GraphQLError::new("JWT auth not available"))?;

        let user_service = DbService::<DBUser>::new(db, "users");

        // Find user by email
        let users = user_service
            .get_records_by_field("email", input.email.clone())
            .await
            .map_err(|e| GraphQLError::new(format!("Database error: {}", e)))?;

        let user = users
            .first()
            .ok_or_else(|| GraphQLError::new("Invalid credentials"))?;

        let is_valid = verify_password(&input.password, &user.password)
            .map_err(|_| GraphQLError::new("Invalid password"))?;

        if !is_valid {
            return Err(GraphQLError::new("Invalid credentials"));
        }

        // Generate JWT token
        let user_id = user
            .id
            .as_ref()
            .map(|thing| thing.id.to_string())
            .unwrap_or_else(|| "unknown".to_string());

        let token = jwt_auth
            .generate_token(&user_id, &format!("{:?}", user.role), &user.address)
            .map_err(|_| GraphQLError::new("Failed to generate token"))?;

        let api_user = User::from_db(user.clone());

        Ok(LoginResponse {
            token,
            user: api_user,
        })
    }
}
