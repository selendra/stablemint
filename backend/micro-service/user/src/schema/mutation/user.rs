use async_graphql::{Context, Object, Result};
use std::sync::Arc;
use tracing::{error, info};

use app_error::AppError;
use app_models::user::{AuthResponse, LoginInput, RegisterInput};

use crate::service::{AuthService, AuthServiceTrait};

// New wallet service type for cross-service communication
use micro_wallet::service::{WalletService, WalletServiceTrait};

pub struct UserMutation;

#[Object]
impl UserMutation {
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

        // Register the user
        let auth_response = auth_service.register(input).await?;

        // Try to get wallet service and create a wallet for the new user
        if let Ok(wallet_service) = ctx.data::<Arc<WalletService>>() {
            match wallet_service
                .create_wallet(&auth_response.user.email)
                .await
            {
                Ok(wallet_info) => {
                    info!("Created wallet for new user: {}", wallet_info.address);
                }
                Err(e) => {
                    // Log the error but don't fail registration if wallet creation fails
                    error!("Failed to create wallet for new user: {}", e);
                }
            }
        } else {
            // This is not a critical error - user is registered but wallet creation will be deferred
            info!("Wallet service not available, wallet will be created on first access");
        }

        Ok(auth_response)
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
