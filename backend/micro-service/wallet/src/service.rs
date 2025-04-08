use app_database::service::DbService;
use app_error::{AppError, AppResult};
use app_models::user::User;
use app_models::wallet::{Wallet, WalletInfo};
use app_utils::generate::EthereumWallet;
use async_trait::async_trait;
use std::sync::Arc;
use tracing::{debug, error, info};

/// Trait defining the wallet service interface
#[async_trait]
pub trait WalletServiceTrait: Send + Sync {
    /// Create a new wallet for a user
    async fn create_wallet(&self, user_email: &str) -> AppResult<WalletInfo>;

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

    /// Get wallet balance (for now returns a placeholder)
    async fn get_balance(&self, wallet_id: &str) -> AppResult<f64>;
    
    /// Associate a wallet with a user by updating the user's wallet_id field
    async fn associate_wallet_with_user(&self, user_id: &str, wallet_id: &str) -> AppResult<()>;
}

/// Implementation of the wallet service
pub struct WalletService {
    wallet_db: Option<Arc<DbService<'static, Wallet>>>,
    pub user_db: Option<Arc<DbService<'static, User>>>,
}

impl WalletService {
    /// Create a new wallet service
    pub fn new() -> Self {
        Self {
            wallet_db: None,
            user_db: None,
        }
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

    /// Helper method to encrypt sensitive wallet data
    /// This is a placeholder for now - in production implement proper encryption
    fn encrypt_sensitive_data(&self, data: &str) -> String {
        // TODO: In production, implement proper encryption
        // For now, just return the plain data with a notice
        format!("ENCRYPTED:{}", data)
    }

    /// Helper method to decrypt sensitive wallet data
    /// This is a placeholder for now - in production implement proper decryption
    fn decrypt_sensitive_data(&self, data: &str, _pin: &str) -> AppResult<String> {
        // TODO: In production, implement proper decryption with PIN verification
        // For now, just return the plain data if it has our fake encryption prefix
        if data.starts_with("ENCRYPTED:") {
            Ok(data.replace("ENCRYPTED:", ""))
        } else {
            Err(AppError::ValidationError(
                "Invalid encrypted format".to_string(),
            ))
        }
    }
}

#[async_trait]
impl WalletServiceTrait for WalletService {
    async fn create_wallet(&self, user_email: &str) -> AppResult<WalletInfo> {
        // Validate user exists
        self.validate_user_exists(user_email).await?;

        // Check if wallet already exists
        if let Some(existing_wallet) = self.check_wallet_exists(user_email).await? {
            return Ok(WalletInfo::from(existing_wallet));
        }

        // Generate new Ethereum wallet
        let eth_wallet = EthereumWallet::new();

        // Extract wallet data
        let address = eth_wallet.address().to_string();
        let private_key = self.encrypt_sensitive_data(&eth_wallet.private_key_hex());
        let mnemonic = self.encrypt_sensitive_data(&eth_wallet.mnemonic_phrase());

        // Create new wallet record
        let wallet = Wallet::new(
            user_email.to_string(),
            address.clone(),
            private_key,
            mnemonic,
        );

        // Store wallet if database is available
        if let Some(wallet_db) = &self.wallet_db {
            info!("Creating new wallet for user: {}", user_email);

            match wallet_db.create_record(wallet.clone()).await {
                Ok(Some(stored)) => Ok(WalletInfo::from(stored)),
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
        // Validate PIN format (in production, this would be more sophisticated)
        if pin.len() != 6 || !pin.chars().all(|c| c.is_digit(10)) {
            return Err(AppError::ValidationError(
                "PIN must be 6 digits".to_string(),
            ));
        }

        // Validate amount
        if amount <= 0.0 {
            return Err(AppError::ValidationError(
                "Amount must be greater than 0".to_string(),
            ));
        }

        // Get source wallet
        let wallet = if let Some(wallet_db) = &self.wallet_db {
            wallet_db
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
                })?
        } else {
            return Err(AppError::ServerError(anyhow::anyhow!(
                "Wallet database not available"
            )));
        };

        // Placeholder for balance check
        // In production, you would check the actual blockchain balance
        let balance = 10.0; // Placeholder balance
        if amount > balance {
            return Err(AppError::ValidationError("Insufficient funds".to_string()));
        }

        // Decrypt private key using PIN
        let _private_key = self.decrypt_sensitive_data(&wallet.private_key, pin)?;

        // This is where you would use the private key to sign and broadcast the transaction
        // For now, we'll just return a placeholder transaction hash
        let transaction_hash = format!("0x{}", hex::encode(uuid::Uuid::new_v4().as_bytes()));

        info!(
            "Transfer of {} from {} to {} initiated",
            amount, wallet.address, to_address
        );

        // In a real implementation, you would monitor the transaction status
        // and update the database accordingly

        Ok(transaction_hash)
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
                .ok_or_else(|| AppError::NotFoundError(format!("User with ID '{}' not found", clean_id)))?;
            
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
}

// For testing purposes
#[cfg(test)]
pub mod mocks {
    use super::*;
    use chrono::Utc;
    use std::sync::{Arc, Mutex};

    pub struct MockWalletService {
        wallets: Arc<Mutex<Vec<Wallet>>>,
        users: Arc<Mutex<Vec<User>>>,
    }

    impl MockWalletService {
        pub fn _new() -> Self {
            Self {
                wallets: Arc::new(Mutex::new(Vec::new())),
                users: Arc::new(Mutex::new(Vec::new())),
            }
        }

        // Add a user for testing
        pub fn _add_user(&self, user: User) {
            self.users.lock().unwrap().push(user);
        }
    }

    #[async_trait]
    impl WalletServiceTrait for MockWalletService {
        async fn create_wallet(&self, user_email: &str) -> AppResult<WalletInfo> {
            // Check if user exists
            let users = self.users.lock().unwrap();
            if !users.iter().any(|u| u.email == user_email) {
                return Err(AppError::NotFoundError(format!(
                    "User with email '{}' not found",
                    user_email
                )));
            }

            // Check if wallet already exists
            let wallets = self.wallets.lock().unwrap();
            if let Some(existing) = wallets.iter().find(|w| w.user_email == user_email) {
                return Ok(WalletInfo::from(existing.clone()));
            }

            drop(wallets);

            // Create new wallet
            let address = format!("0x{}", hex::encode(&[1, 2, 3, 4, 5, 6, 7, 8, 9, 10]));
            let private_key = "ENCRYPTED:mock_private_key".to_string();
            let mnemonic = "ENCRYPTED:mock mnemonic phrase".to_string();

            let wallet = Wallet {
                id: Wallet::generate_id(),
                user_email: user_email.to_string(),
                address: address.clone(),
                private_key,
                mnemonic,
                created_at: Utc::now(),
                updated_at: Utc::now(),
            };

            let wallet_info = WalletInfo::from(wallet.clone());
            self.wallets.lock().unwrap().push(wallet);

            Ok(wallet_info)
        }

        async fn get_wallet_by_user_email(&self, user_email: &str) -> AppResult<WalletInfo> {
            let wallets = self.wallets.lock().unwrap();
            if let Some(wallet) = wallets.iter().find(|w| w.user_email == user_email) {
                Ok(WalletInfo::from(wallet.clone()))
            } else {
                Err(AppError::NotFoundError(format!(
                    "Wallet not found for user: {}",
                    user_email
                )))
            }
        }

        async fn get_wallet_by_id(&self, wallet_id: &str) -> AppResult<WalletInfo> {
            let wallets = self.wallets.lock().unwrap();
            if let Some(wallet) = wallets.iter().find(|w| w.id.id.to_string() == wallet_id) {
                Ok(WalletInfo::from(wallet.clone()))
            } else {
                Err(AppError::NotFoundError(format!(
                    "Wallet with ID '{}' not found",
                    wallet_id
                )))
            }
        }

        async fn transfer(
            &self,
            _from_wallet_id: &str,
            _to_address: &str,
            amount: f64,
            pin: &str,
        ) -> AppResult<String> {
            // Mock validation
            if pin.len() != 6 || !pin.chars().all(|c| c.is_digit(10)) {
                return Err(AppError::ValidationError(
                    "PIN must be 6 digits".to_string(),
                ));
            }

            if amount <= 0.0 {
                return Err(AppError::ValidationError(
                    "Amount must be greater than 0".to_string(),
                ));
            }

            // Mock transaction hash
            Ok(format!(
                "0x{}",
                hex::encode(uuid::Uuid::new_v4().as_bytes())
            ))
        }

        async fn get_balance(&self, wallet_id: &str) -> AppResult<f64> {
            // Check if wallet exists
            let wallets = self.wallets.lock().unwrap();
            if wallets.iter().any(|w| w.id.id.to_string() == wallet_id) {
                Ok(10.0) // Mock balance
            } else {
                Err(AppError::NotFoundError(format!(
                    "Wallet with ID '{}' not found",
                    wallet_id
                )))
            }
        }
        
        async fn associate_wallet_with_user(&self, user_id: &str, wallet_id: &str) -> AppResult<()> {
            let mut users = self.users.lock().unwrap();
            if let Some(user) = users.iter_mut().find(|u| u.id.id.to_string() == user_id) {
                user.wallet_id = Some(wallet_id.to_string());
                user.updated_at = Utc::now();
                Ok(())
            } else {
                Err(AppError::NotFoundError(format!(
                    "User with ID '{}' not found",
                    user_id
                )))
            }
        }
    }
}