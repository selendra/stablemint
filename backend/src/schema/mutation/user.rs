use crate::{
    error::AppError,
    handlers::auth::AuthService,
    models::user::{AuthResponse, LoginInput, RegisterInput},
};
use async_graphql::{Context, Object, Result};
use std::sync::Arc;
use tracing::error;

pub struct Mutation;

#[Object]
impl Mutation {
    // Register a new user
    async fn register(
        &self,
        ctx: &Context<'_>,
        input: RegisterInput,
    ) -> Result<AuthResponse, AppError> {
        // Try to get auth service from context with better error handling
        let auth_service = match ctx.data::<Arc<AuthService>>() {
            Ok(service) => service,
            Err(e) => {
                error!("Failed to get auth service: {:?}", e);
                return Err(AppError::ServerError(anyhow::anyhow!(
                    "Auth service not available"
                )));
            }
        };

        auth_service.register(input).await
    }

    // Login user
    async fn login(&self, ctx: &Context<'_>, input: LoginInput) -> Result<AuthResponse, AppError> {
        // Try to get auth service from context with better error handling
        let auth_service = match ctx.data::<Arc<AuthService>>() {
            Ok(service) => service,
            Err(e) => {
                error!("Failed to get auth service: {:?}", e);
                return Err(AppError::ServerError(anyhow::anyhow!(
                    "Auth service not available"
                )));
            }
        };

        auth_service.login(input).await
    }
}
