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
    pub wallet_id: String,
    pub to_address: String,
    pub amount: f64,
    pub pin: String,
}

pub struct WalletMutation;

#[Object]
impl WalletMutation {
    // Create a wallet for the current user
    async fn create_wallet(&self, ctx: &Context<'_>) -> Result<WalletInfo, AppError> {
        // Get the claims from the context
        let claims = ctx.data::<Claims>().map_err(|_| {
            AppError::AuthenticationError("Authentication required to create a wallet".to_string())
        })?;

        // Get the wallet service
        let wallet_service = ctx.data::<Arc<WalletService>>().map_err(|e| {
            error!("Failed to get wallet service: {:?}", e);
            AppError::ServerError(anyhow::anyhow!("Wallet service not available"))
        })?;

        // Get user by ID from the claims
        let user = wallet_service.get_user_by_id(&claims.sub).await?;


        // Create wallet for the user
        let wallet_info = wallet_service.create_wallet(&user.email).await?;

        wallet_service.associate_wallet_with_user(&claims.sub, &wallet_info.id).await?;

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

        // Get the wallet
        let wallet = wallet_service.get_wallet_by_id(&input.wallet_id).await?;

        // Verify ownership
        if wallet.user_email != user.email {
            return Err(AppError::AuthorizationError(
                "You do not have permission to transfer from this wallet".to_string(),
            ));
        }

        // Perform the transfer
        wallet_service
            .transfer(
                &input.wallet_id,
                &input.to_address,
                input.amount,
                &input.pin,
            )
            .await
    }
}
