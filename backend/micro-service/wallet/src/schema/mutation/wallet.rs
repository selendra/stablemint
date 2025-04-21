use async_graphql::{Context, InputObject, Object, Result};
use std::sync::Arc;
use tracing::error;

use app_error::AppError;
use app_middleware::Claims;
use app_models::wallet::WalletInfo;

use crate::middleware::validate_pin;
use crate::service::{WalletService, WalletServiceTrait};

#[derive(InputObject)]
pub struct TransferInput {
    pub to_address: String,
    pub amount: f64,
    pub pin: String,
}

#[derive(InputObject)]
pub struct CreateWalletInput {
    pub pin: String,
}

#[derive(InputObject)]
pub struct ChangePinInput {
    pub old_pin: String,
    pub new_pin: String,
}

pub struct WalletMutation;

#[Object]
impl WalletMutation {
    // Create a wallet for the current user
    async fn create_wallet(
        &self,
        ctx: &Context<'_>,
        input: CreateWalletInput,
    ) -> Result<WalletInfo, AppError> {
        // Get the claims from the context
        let claims = ctx.data::<Claims>().map_err(|_| {
            AppError::AuthenticationError("Authentication required to create a wallet".to_string())
        })?;

        // Validate PIN
        validate_pin(&input.pin)?;

        // Get the wallet service
        let wallet_service = ctx.data::<Arc<WalletService>>().map_err(|e| {
            error!("Failed to get wallet service: {:?}", e);
            AppError::ServerError(anyhow::anyhow!("Wallet service not available"))
        })?;

        // Get user by ID from the claims
        let user = wallet_service.get_user_by_id(&claims.sub).await?;

        // Create wallet for the user with PIN
        let wallet_info = wallet_service
            .create_wallet(&user.email, &input.pin)
            .await?;

        wallet_service
            .associate_wallet_with_user(&claims.sub, &wallet_info.id)
            .await?;

        Ok(wallet_info)
    }

    // Transfer funds from wallet (requires PIN)
    async fn transfer(&self, ctx: &Context<'_>, input: TransferInput) -> Result<String, AppError> {
        // Get the claims from the context
        let claims = ctx.data::<Claims>().map_err(|_| {
            AppError::AuthenticationError("Authentication required to transfer funds".to_string())
        })?;

        // Get the wallet service
        let wallet_service = ctx.data::<Arc<WalletService>>().map_err(|e| {
            error!("Failed to get wallet service: {:?}", e);
            AppError::ServerError(anyhow::anyhow!("Wallet service not available"))
        })?;

        // Validate PIN format
        validate_pin(&input.pin)?;

        // Get user by ID from the claims
        let user = wallet_service.get_user_by_id(&claims.sub).await?;

        // Get the user's wallet
        let wallet = wallet_service.get_wallet_by_user_email(&user.email).await?;

        // Verify the PIN is correct before proceeding with transfer
        let is_pin_valid = wallet_service.verify_pin(&wallet.id, &input.pin).await?;
        if !is_pin_valid {
            return Err(AppError::AuthenticationError(
                "Invalid PIN. Transfer canceled for security reasons.".to_string(),
            ));
        }

        // Perform the transfer
        wallet_service
            .transfer(&wallet.id, &input.to_address, input.amount, &input.pin)
            .await
    }

    // Change wallet PIN
    async fn change_wallet_pin(
        &self,
        ctx: &Context<'_>,
        input: ChangePinInput,
    ) -> Result<bool, AppError> {
        // Get the claims from the context
        let claims = ctx.data::<Claims>().map_err(|_| {
            AppError::AuthenticationError(
                "Authentication required to change wallet PIN".to_string(),
            )
        })?;

        // Get the wallet service
        let wallet_service = ctx.data::<Arc<WalletService>>().map_err(|e| {
            error!("Failed to get wallet service: {:?}", e);
            AppError::ServerError(anyhow::anyhow!("Wallet service not available"))
        })?;

        // Get user by ID from the claims
        let user = wallet_service.get_user_by_id(&claims.sub).await?;

        // Get the user's wallet
        let wallet = wallet_service.get_wallet_by_user_email(&user.email).await?;

        // Verify the old PIN is correct before allowing PIN change
        let is_pin_valid = wallet_service
            .verify_pin(&wallet.id, &input.old_pin)
            .await?;
        if !is_pin_valid {
            return Err(AppError::AuthenticationError(
                "Current PIN is incorrect. PIN change canceled for security reasons.".to_string(),
            ));
        }

        // Change the PIN
        wallet_service
            .change_wallet_pin(&wallet.id, &input.old_pin, &input.new_pin)
            .await?;

        Ok(true)
    }

    // Verify wallet PIN (useful for client-side validation)
    async fn verify_wallet_pin(&self, ctx: &Context<'_>, pin: String) -> Result<bool, AppError> {
        // Get the claims from the context
        let claims = ctx.data::<Claims>().map_err(|_| {
            AppError::AuthenticationError(
                "Authentication required to verify wallet PIN".to_string(),
            )
        })?;

        // Get the wallet service
        let wallet_service = ctx.data::<Arc<WalletService>>().map_err(|e| {
            error!("Failed to get wallet service: {:?}", e);
            AppError::ServerError(anyhow::anyhow!("Wallet service not available"))
        })?;

        // Get user by ID from the claims
        let user = wallet_service.get_user_by_id(&claims.sub).await?;

        // Get the user's wallet
        let wallet = wallet_service.get_wallet_by_user_email(&user.email).await?;

        // Verify the PIN
        wallet_service.verify_pin(&wallet.id, &pin).await
    }
}
