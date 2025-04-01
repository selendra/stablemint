use std::sync::Arc;
use async_graphql::{Object, Context};
use stablemint_error::AppError;
use stablemint_models::user::{CreateUserInput, User, UserRole};
use stablemint_surrealdb::types:: Database;

use crate::schema::UserService;

pub struct MutationRoot;

#[Object]
impl MutationRoot {
    // Create a new user
    async fn create_user(&self, ctx: &Context<'_>, input: CreateUserInput) -> Result<User, AppError> {
        // Only admins can create admin users
        if input.role == UserRole::Admin {
            if let Some(role) = ctx.data_opt::<UserRole>() {
                if *role != UserRole::Admin {
                    return Err(AppError::Unauthorized(anyhow::anyhow!("Admin access required")));
                }
            } else {
                return Err(AppError::Unauthorized(anyhow::anyhow!("Authentication required")));
            }
        }
        
        let user_service = UserService::new(ctx.data::<Arc<Database>>().unwrap());
        user_service.create_user(input).await.map_err(|e| {
            AppError::Database(anyhow::anyhow!("Failed to create user: {}", e))
        })
    }
}