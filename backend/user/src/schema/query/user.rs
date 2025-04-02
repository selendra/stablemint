use async_graphql::{Context, Object, Result};
use std::sync::Arc;

use app_authentication::service::AuthServiceTrait;
use app_authentication::{AuthService, Claims};
use app_error::AppError;
use app_models::user::UserProfile;

pub struct UserQuery;

#[Object]
impl UserQuery {
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
