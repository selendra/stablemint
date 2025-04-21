mod keys;

use app_database::service::DbService;
use app_error::{AppError, AppResult};
use app_models::WalletKey;
use app_models::user::User;
use app_models::wallet::{Wallet, WalletInfo};
use app_utils::crypto::WalletEncryptionService;
use app_utils::generate::EthereumWallet;
use async_trait::async_trait;
use std::sync::Arc;
use tracing::{debug, error, info};

/// Trait defining the wallet service interface
#[async_trait]
pub trait WalletServiceTrait: Send + Sync {
    /// Create a new wallet for a user with PIN
    async fn create_wallet(&self, user_email: &str, pin: &str) -> AppResult<WalletInfo>;

    /// Get a wallet by user email
    async fn get_wallet_by_user_email(&self, user_email: &str) -> AppResult<WalletInfo>;

    /// Get a wallet by ID
    async fn get_wallet_by_id(&self, wallet_id: &str) -> AppResult<WalletInfo>;

    /// Transfer funds from a wallet (requires PIN)
    async fn transfer(
        &self,
        from_wallet_id: &str,
        to_address: &str,
        amount: f64,
        pin: &str,
    ) -> AppResult<String>;

    /// Get wallet balance
    async fn get_balance(&self, wallet_id: &str) -> AppResult<f64>;

    /// Associate a wallet with a user
    async fn associate_wallet_with_user(&self, user_id: &str, wallet_id: &str) -> AppResult<()>;

    /// Change wallet PIN
    async fn change_wallet_pin(
        &self,
        wallet_id: &str,
        old_pin: &str,
        new_pin: &str,
    ) -> AppResult<()>;

    /// Verify wallet PIN
    async fn verify_pin(&self, wallet_id: &str, pin: &str) -> AppResult<bool>;
}

/// Implementation of the wallet service
pub struct WalletService {
    wallet_db: Option<Arc<DbService<'static, Wallet>>>,
    wallet_key_db: Option<Arc<DbService<'static, WalletKey>>>, // New field for wallet keys
    pub user_db: Option<Arc<DbService<'static, User>>>,
    encryption_service: Arc<WalletEncryptionService>,
}

impl WalletService {
    /// Create a new wallet service
    pub fn new(encryption_service: Arc<WalletEncryptionService>) -> Self {
        Self {
            wallet_db: None,
            wallet_key_db: None, // Initialize as None
            user_db: None,
            encryption_service,
        }
    }

    async fn get_private_key(&self, wallet_id: &str, pin: &str) -> AppResult<String> {
        // Validate PIN format
        Self::validate_pin(pin)?;

        // Get the encrypted data
        let encrypted_data = self.get_wallet_encrypted_data(wallet_id).await?;

        // Decrypt the private key
        self.encryption_service
            .decrypt_private_key(&encrypted_data, pin)
            .await
    }

    /// Add a wallet database service
    pub fn with_wallet_db(mut self, wallet_db: Arc<DbService<'static, Wallet>>) -> Self {
        self.wallet_db = Some(wallet_db);
        self
    }

    /// Add a user database service for validation
    pub fn with_user_db(mut self, user_db: Arc<DbService<'static, User>>) -> Self {
        self.user_db = Some(user_db);
        self
    }

    /// Add a wallet key database service
    pub fn with_wallet_key_db(mut self, wallet_key_db: Arc<DbService<'static, WalletKey>>) -> Self {
        self.wallet_key_db = Some(wallet_key_db);
        self
    }

    /// Helper method to validate user exists
    async fn validate_user_exists(&self, user_email: &str) -> AppResult<User> {
        if let Some(user_db) = &self.user_db {
            let users = user_db
                .get_records_by_field("email", user_email.to_string())
                .await
                .map_err(|e| {
                    error!("Database error when checking for user: {}", e);
                    AppError::DatabaseError(anyhow::anyhow!(e))
                })?;

            if users.is_empty() {
                return Err(AppError::NotFoundError(format!(
                    "User with email '{}' not found",
                    user_email
                )));
            }

            Ok(users[0].clone())
        } else {
            Err(AppError::ServerError(anyhow::anyhow!(
                "User database not available"
            )))
        }
    }

    /// Helper method to check if wallet already exists for a user
    async fn check_wallet_exists(&self, user_email: &str) -> AppResult<Option<Wallet>> {
        if let Some(wallet_db) = &self.wallet_db {
            let wallets = wallet_db
                .get_records_by_field("user_email", user_email.to_string())
                .await
                .map_err(|e| {
                    error!("Database error when checking for existing wallet: {}", e);
                    AppError::DatabaseError(anyhow::anyhow!(e))
                })?;

            if !wallets.is_empty() {
                return Ok(Some(wallets[0].clone()));
            }
        } else {
            return Err(AppError::ServerError(anyhow::anyhow!(
                "Wallet database not available"
            )));
        }

        Ok(None)
    }

    /// Helper method to validate PIN format
    fn validate_pin(pin: &str) -> AppResult<()> {
        if pin.len() != 6 || !pin.chars().all(|c| c.is_digit(10)) {
            return Err(AppError::ValidationError(
                "PIN must be a 6-digit number".to_string(),
            ));
        }
        Ok(())
    }
}

#[async_trait]
impl WalletServiceTrait for WalletService {
    async fn create_wallet(&self, user_email: &str, pin: &str) -> AppResult<WalletInfo> {
        // Validate PIN format
        Self::validate_pin(pin)?;

        // Validate user exists
        let user = self.validate_user_exists(user_email).await?;

        // Check if wallet already exists
        if let Some(existing_wallet) = self.check_wallet_exists(user_email).await? {
            return Ok(WalletInfo::from(existing_wallet));
        }

        // Generate new Ethereum wallet
        let eth_wallet = EthereumWallet::new();

        // Extract wallet data
        let address = eth_wallet.address().to_string();
        let private_key = eth_wallet.private_key_hex();

        // Encrypt private key with PIN and system encryption
        let encrypted_data = self
            .encryption_service
            .encrypt_private_key(&private_key, pin)
            .await?;

        // Create new wallet record (without private key)
        let wallet = Wallet::new(user_email.to_string(), address.clone());

        // Store wallet if database is available
        if let Some(wallet_db) = &self.wallet_db {
            info!("Creating new wallet for user: {}", user_email);

            match wallet_db.create_record(wallet.clone()).await {
                Ok(Some(stored)) => {
                    // Get the wallet ID
                    let wallet_id = stored.id.id.to_string();

                    // Store the encrypted key separately
                    match self.store_wallet_key(&wallet_id, &encrypted_data).await {
                        Ok(key_id) => {
                            // Update the wallet with the key ID reference
                            let mut updated_wallet = stored.clone();
                            updated_wallet.key_id = Some(key_id);
                            updated_wallet.updated_at = chrono::Utc::now();

                            // Save the updated wallet with key reference
                            match wallet_db
                                .update_record(&wallet_id, updated_wallet.clone())
                                .await
                            {
                                Ok(_) => {
                                    // Associate wallet with user
                                    if let Some(user_db) = &self.user_db {
                                        let mut updated_user = user.clone();
                                        updated_user.wallet_id = Some(wallet_id.clone());
                                        updated_user.updated_at = chrono::Utc::now();

                                        // Update user record with wallet reference
                                        let _ = user_db
                                            .update_record(&user.id.id.to_string(), updated_user)
                                            .await;
                                    }

                                    Ok(WalletInfo::from(updated_wallet))
                                }
                                Err(e) => {
                                    error!("Failed to update wallet with key ID: {}", e);
                                    Err(AppError::DatabaseError(anyhow::anyhow!(e)))
                                }
                            }
                        }
                        Err(e) => {
                            error!("Failed to store wallet key: {}", e);
                            Err(e)
                        }
                    }
                }
                Ok(None) => {
                    error!("Database did not return stored wallet");
                    Ok(WalletInfo::from(wallet)) // Use the original wallet as fallback
                }
                Err(e) => {
                    error!("Failed to store wallet in database: {}", e);
                    Err(AppError::DatabaseError(anyhow::anyhow!(e)))
                }
            }
        } else {
            error!("Wallet database not available for storing wallet");
            Ok(WalletInfo::from(wallet))
        }
    }

    async fn get_wallet_by_user_email(&self, user_email: &str) -> AppResult<WalletInfo> {
        // Check if wallet exists
        match self.check_wallet_exists(user_email).await? {
            Some(wallet) => Ok(WalletInfo::from(wallet)),
            None => Err(AppError::NotFoundError(format!(
                "Wallet not found for user: {}",
                user_email
            ))),
        }
    }

    async fn get_wallet_by_id(&self, wallet_id: &str) -> AppResult<WalletInfo> {
        if let Some(wallet_db) = &self.wallet_db {
            let wallet = wallet_db
                .get_record_by_id(wallet_id)
                .await
                .map_err(|e| {
                    error!("Database error when fetching wallet by ID: {}", e);
                    AppError::DatabaseError(anyhow::anyhow!(e))
                })?
                .ok_or_else(|| {
                    AppError::NotFoundError(format!("Wallet with ID '{}' not found", wallet_id))
                })?;

            Ok(WalletInfo::from(wallet))
        } else {
            Err(AppError::ServerError(anyhow::anyhow!(
                "Wallet database not available"
            )))
        }
    }

    async fn transfer(
        &self,
        from_wallet_id: &str,
        to_address: &str,
        amount: f64,
        pin: &str,
    ) -> AppResult<String> {
        // Validate PIN format
        Self::validate_pin(pin)?;

        // Validate amount
        if amount <= 0.0 {
            return Err(AppError::ValidationError(
                "Amount must be greater than 0".to_string(),
            ));
        }

        // Get source wallet
        if let Some(wallet_db) = &self.wallet_db {
            let wallet = wallet_db
                .get_record_by_id(from_wallet_id)
                .await
                .map_err(|e| {
                    error!("Database error when fetching wallet for transfer: {}", e);
                    AppError::DatabaseError(anyhow::anyhow!(e))
                })?
                .ok_or_else(|| {
                    AppError::NotFoundError(format!(
                        "Wallet with ID '{}' not found",
                        from_wallet_id
                    ))
                })?;

            // Placeholder for balance check
            // In production, you would check the actual blockchain balance
            let balance = 10.0; // Placeholder balance
            if amount > balance {
                return Err(AppError::ValidationError("Insufficient funds".to_string()));
            }

            // Verify the PIN is correct before proceeding with transfer
            let is_pin_valid = self.verify_pin(from_wallet_id, pin).await?;
            if !is_pin_valid {
                return Err(AppError::AuthenticationError(
                    "Invalid PIN. Transfer canceled for security reasons.".to_string(),
                ));
            }

            // Get the private key for transaction signing
            let _private_key = self.get_private_key(from_wallet_id, pin).await?;

            // This is where you would use the private key to sign and broadcast the transaction
            debug!("Successfully decrypted private key for transaction signing");

            // For now, just return a placeholder transaction hash
            let transaction_hash = format!("0x{}", hex::encode(uuid::Uuid::new_v4().as_bytes()));

            info!(
                "Transfer of {} from {} to {} initiated",
                amount, wallet.address, to_address
            );

            // In a real implementation, you would monitor the transaction status
            // and update the database accordingly

            Ok(transaction_hash)
        } else {
            Err(AppError::ServerError(anyhow::anyhow!(
                "Wallet database not available"
            )))
        }
    }

    async fn get_balance(&self, wallet_id: &str) -> AppResult<f64> {
        // Get wallet
        if let Some(wallet_db) = &self.wallet_db {
            let wallet = wallet_db
                .get_record_by_id(wallet_id)
                .await
                .map_err(|e| {
                    error!("Database error when fetching wallet for balance: {}", e);
                    AppError::DatabaseError(anyhow::anyhow!(e))
                })?
                .ok_or_else(|| {
                    AppError::NotFoundError(format!("Wallet with ID '{}' not found", wallet_id))
                })?;

            debug!("Getting balance for wallet address: {}", wallet.address);

            // In a real implementation, you would fetch the actual balance from the blockchain
            // For now, return a placeholder value
            Ok(10.0)
        } else {
            Err(AppError::ServerError(anyhow::anyhow!(
                "Wallet database not available"
            )))
        }
    }

    async fn associate_wallet_with_user(&self, user_id: &str, wallet_id: &str) -> AppResult<()> {
        if let Some(user_db) = &self.user_db {
            // Clean the user ID (remove surrounding angle brackets if present)
            let clean_id = user_id
                .trim_start_matches('⟨')
                .trim_end_matches('⟩')
                .to_string();

            // Get the user
            let mut user = user_db
                .get_record_by_id(&clean_id)
                .await
                .map_err(|e| {
                    error!("Database error when fetching user: {}", e);
                    AppError::DatabaseError(anyhow::anyhow!(e))
                })?
                .ok_or_else(|| {
                    AppError::NotFoundError(format!("User with ID '{}' not found", clean_id))
                })?;

            // Update the wallet_id field
            user.wallet_id = Some(wallet_id.to_string());
            user.updated_at = chrono::Utc::now();

            // Save the updated user
            user_db.update_record(&clean_id, user).await.map_err(|e| {
                error!("Failed to update user with wallet ID: {}", e);
                AppError::DatabaseError(anyhow::anyhow!(e))
            })?;

            info!("Associated wallet {} with user {}", wallet_id, clean_id);
            Ok(())
        } else {
            Err(AppError::ServerError(anyhow::anyhow!(
                "User database not available"
            )))
        }
    }

    async fn verify_pin(&self, wallet_id: &str, pin: &str) -> AppResult<bool> {
        // Validate PIN format
        Self::validate_pin(pin)?;

        // Get the encrypted data
        match self.get_wallet_encrypted_data(wallet_id).await {
            Ok(encrypted_data) => {
                // Try to decrypt with PIN - we don't need the result, just whether it succeeds
                match self
                    .encryption_service
                    .decrypt_private_key(&encrypted_data, pin)
                    .await
                {
                    Ok(_) => Ok(true),
                    Err(_) => Ok(false),
                }
            }
            Err(e) => {
                // If it's just that the key doesn't exist, return false
                if let AppError::NotFoundError(_) = e {
                    return Ok(false);
                }
                // Otherwise propagate the error
                Err(e)
            }
        }
    }

    async fn change_wallet_pin(
        &self,
        wallet_id: &str,
        old_pin: &str,
        new_pin: &str,
    ) -> AppResult<()> {
        // Validate both PINs
        Self::validate_pin(old_pin)?;
        Self::validate_pin(new_pin)?;

        // Get private key using old PIN
        let private_key = self.get_private_key(wallet_id, old_pin).await?;

        // Re-encrypt with new PIN
        let new_encrypted_data = self
            .encryption_service
            .encrypt_private_key(&private_key, new_pin)
            .await?;

        // Update the wallet key
        self.update_wallet_key(wallet_id, &new_encrypted_data).await
    }
}
