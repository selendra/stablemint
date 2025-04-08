use app_error::AppError;
use app_middleware::Claims;
use axum::{
    extract::{Request, State},
    middleware::Next,
    response::Response,
};
use std::sync::Arc;
use tracing::{debug, warn};

use crate::service::{WalletService, WalletServiceTrait};

/// Middleware to verify that the user is the owner of the requested wallet
pub async fn wallet_owner_middleware(
    claims: Option<Claims>,
    State(wallet_service): State<Arc<WalletService>>,
    req: Request,
    next: Next,
) -> Result<Response, AppError> {
    // Extract the wallet_id from the request path
    let path = req.uri().path();
    let wallet_id = path
        .split('/')
        .find(|segment| segment.starts_with("wallets:"))
        .map(|s| s.to_string());

    // If there's no wallet ID in the path, just continue
    if wallet_id.is_none() {
        return Ok(next.run(req).await);
    }

    let wallet_id = wallet_id.unwrap();
    debug!("Wallet access check for wallet_id: {}", wallet_id);

    // Extract user information from JWT claims
    match claims {
        Some(claims) => {
            // Get the wallet
            let wallet_info = match wallet_service.get_wallet_by_id(&wallet_id).await {
                Ok(wallet) => wallet,
                Err(err) => {
                    if let AppError::NotFoundError(_) = &err {
                        // If wallet doesn't exist, just continue and let the handler deal with it
                        return Ok(next.run(req).await);
                    }
                    return Err(err);
                }
            };

            // Check if the authenticated user is the wallet owner
            // First try to get user by ID, then check email matches wallet's user_email
            match wallet_service.get_user_by_id(&claims.sub).await {
                Ok(user) => {
                    if user.email != wallet_info.user_email {
                        warn!(
                            "Access denied: User {} attempted to access wallet {}",
                            claims.username, wallet_id
                        );
                        return Err(AppError::AuthorizationError(
                            "You do not have permission to access this wallet".to_string(),
                        ));
                    }
                    // User is the wallet owner, continue
                    debug!(
                        "Access granted: User {} owns wallet {}",
                        claims.username, wallet_id
                    );
                    Ok(next.run(req).await)
                }
                Err(_) => {
                    warn!("User validation failed for user ID: {}", claims.sub);
                    Err(AppError::AuthorizationError(
                        "Authentication validation failed".to_string(),
                    ))
                }
            }
        }

        None => {
            warn!("Unauthenticated access attempt to wallet: {}", wallet_id);
            Err(AppError::AuthenticationError(
                "Authentication required to access wallet".to_string(),
            ))
        }
    }
}

// Helper function to validate PIN format
pub fn validate_pin(pin: &str) -> Result<(), AppError> {
    if pin.len() != 6 || !pin.chars().all(|c| c.is_digit(10)) {
        return Err(AppError::ValidationError(
            "PIN must be a 6-digit number".to_string(),
        ));
    }
    Ok(())
}

/// Extend the WalletService to add user validation
impl WalletService {
    /// Get a user by ID
    pub async fn get_user_by_id(&self, user_id: &str) -> Result<app_models::user::User, AppError> {
        if let Some(user_db) = &self.user_db {
            let clean_id = user_id
                .trim_start_matches('⟨')
                .trim_end_matches('⟩')
                .to_string();

            let user = user_db
                .get_record_by_id(&clean_id)
                .await
                .map_err(|e| {
                    tracing::error!("Database error when fetching user by ID: {}", e);
                    AppError::DatabaseError(anyhow::anyhow!(e))
                })?
                .ok_or_else(|| AppError::NotFoundError("User not found".to_string()))?;

            Ok(user)
        } else {
            Err(AppError::ServerError(anyhow::anyhow!(
                "User database not available"
            )))
        }
    }
}
