use async_graphql::{Context, FieldError, Object, Result};
use std::sync::Arc;

use app_authentication::service::AuthServiceTrait;
use app_authentication::{AuthService, Claims};
use app_error::AppError;
use app_models::user::UserProfile;

pub struct UserQuery;

#[Object]
impl UserQuery {
    // Get the current user's profile (requires auth)
    async fn me(&self, ctx: &Context<'_>) -> Result<UserProfile, FieldError> {
        // Get the claims from the context
        let claims = ctx.data::<Claims>().map_err(|_| {
            AppError::AuthenticationError(
                "Authentication required. Please log in to view your profile.".to_string(),
            )
            .to_field_error()
        })?;

        // Get the auth service
        let auth_service = ctx.data::<Arc<AuthService>>().map_err(|_| {
            AppError::ServerError(anyhow::anyhow!(
                "Internal configuration error: Auth service not available"
            ))
            .to_field_error()
        })?;

        // Get user by ID from the claims
        auth_service
            .get_user_by_id(&claims.sub)
            .await
            .map_err(|err| err.to_field_error())
    }
}
