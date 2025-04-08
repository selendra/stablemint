use async_graphql::{Context, FieldError, Object, Result};
use std::sync::Arc;

use app_error::AppError;
use app_middleware::Claims;
use app_models::wallet::WalletInfo;

use crate::service::{WalletService, WalletServiceTrait};

pub struct WalletQuery;

#[Object]
impl WalletQuery {
    // Get the current user's wallet (requires auth)
    async fn my_wallet(&self, ctx: &Context<'_>) -> Result<WalletInfo, FieldError> {
        // Get the claims from the context
        let claims = ctx.data::<Claims>().map_err(|_| {
            AppError::AuthenticationError(
                "Authentication required. Please log in to view your wallet.".to_string(),
            )
            .to_field_error()
        })?;

        // Get the wallet service
        let wallet_service = ctx.data::<Arc<WalletService>>().map_err(|_| {
            AppError::ServerError(anyhow::anyhow!(
                "Internal configuration error: Wallet service not available"
            ))
            .to_field_error()
        })?;

        // Get user by ID from the claims
        let user = wallet_service
            .get_user_by_id(&claims.sub)
            .await
            .map_err(|err| err.to_field_error())?;

        // Get wallet by user email
        wallet_service
            .get_wallet_by_user_email(&user.email)
            .await
            .map_err(|err| err.to_field_error())
    }

    // Get wallet balance
    async fn wallet_balance(
        &self,
        ctx: &Context<'_>,
        wallet_id: String,
    ) -> Result<f64, FieldError> {
        // Get the claims from the context
        let claims = ctx.data::<Claims>().map_err(|_| {
            AppError::AuthenticationError(
                "Authentication required. Please log in to view wallet balance.".to_string(),
            )
            .to_field_error()
        })?;

        // Get the wallet service
        let wallet_service = ctx.data::<Arc<WalletService>>().map_err(|_| {
            AppError::ServerError(anyhow::anyhow!(
                "Internal configuration error: Wallet service not available"
            ))
            .to_field_error()
        })?;

        // Get user by ID from the claims
        let user = wallet_service
            .get_user_by_id(&claims.sub)
            .await
            .map_err(|err| err.to_field_error())?;

        // Get the wallet
        let wallet = wallet_service
            .get_wallet_by_id(&wallet_id)
            .await
            .map_err(|err| err.to_field_error())?;

        // Verify ownership
        if wallet.user_email != user.email {
            return Err(AppError::AuthorizationError(
                "You do not have permission to view this wallet's balance".to_string(),
            )
            .to_field_error());
        }

        // Get the balance
        wallet_service
            .get_balance(&wallet_id)
            .await
            .map_err(|err| err.to_field_error())
    }
}
