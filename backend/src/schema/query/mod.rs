use crate::{
    error::AppError,
    handlers::auth::AuthService,
    middleware::auth::jwt::Claims,
    models::{HelloWorld, user::UserProfile},
};
use async_graphql::{Context, Object, Result};
use std::sync::Arc;

pub struct Query;

#[Object]
impl Query {
    async fn hello_world(&self) -> HelloWorld {
        HelloWorld {
            message: "Hello, World!".to_string(),
        }
    }

    async fn greet(&self, name: Option<String>) -> String {
        match name {
            Some(name) => format!("Hello, {}!", name),
            None => "Hello, anonymous!".to_string(),
        }
    }

    // Get the current user's profile (requires auth)
    async fn me(&self, ctx: &Context<'_>) -> Result<UserProfile, AppError> {
        // Get the claims from the context
        let claims = ctx
            .data::<Claims>()
            .map_err(|_| AppError::AuthenticationError("Not authenticated".to_string()))?;

        // Get the auth service
        let auth_service = ctx
            .data::<Arc<AuthService>>()
            .map_err(|_| AppError::ServerError(anyhow::anyhow!("Auth service not available")))?;

        // Get user by ID from the claims
        auth_service.get_user_by_id(&claims.sub).await
    }
}
