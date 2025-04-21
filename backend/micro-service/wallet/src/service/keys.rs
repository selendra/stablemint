use app_error::{AppError, AppResult};
use app_models::wallet::WalletKey;
use app_utils::crypto::{WalletEncryptedData, WalletEncryptionService};
use std::sync::Arc;
use tracing::{error, info};

use crate::service::WalletService;

/// Extension to WalletService for managing wallet keys
impl WalletService {
    /// Convert WalletEncryptedData to WalletKey
    fn encrypted_data_to_wallet_key(wallet_id: &str, data: &WalletEncryptedData) -> WalletKey {
        WalletKey::new(
            wallet_id.to_string(),
            data.encrypted_private_key.clone(),
            data.encrypted_dek.clone(),
            data.master_key_identifier.clone(),
            data.dek_id.clone(),
            data.algorithm.clone(),
            data.pin_salt.clone(),
            data.pin_iv.clone(),
            data.dek_iv.clone(),
            data.master_iv.clone(),
        )
    }

    /// Convert WalletKey to WalletEncryptedData
    fn wallet_key_to_encrypted_data(key: &WalletKey) -> WalletEncryptedData {
        WalletEncryptedData {
            user_id: "".to_string(), // Not used for decryption
            encrypted_private_key: key.encrypted_private_key.clone(),
            encrypted_dek: key.encrypted_dek.clone(),
            master_key_identifier: key.master_key_id.clone(),
            dek_id: key.dek_id.clone(),
            algorithm: key.algorithm.clone(),
            pin_salt: key.pin_salt.clone(),
            pin_iv: key.pin_iv.clone(),
            dek_iv: key.dek_iv.clone(),
            master_iv: key.master_iv.clone(),
        }
    }

    /// Store a key in the keys table and update the wallet reference
    pub async fn store_wallet_key(
        &self, 
        wallet_id: &str, 
        encrypted_data: &WalletEncryptedData
    ) -> AppResult<String> {
        // Create a new wallet key record
        let wallet_key = Self::encrypted_data_to_wallet_key(wallet_id, encrypted_data);
        let key_id = wallet_key.id.id.to_string();

        // Store the key in the keys table
        if let Some(wallet_key_db) = &self.wallet_key_db {
            info!("Storing encrypted wallet key for wallet: {}", wallet_id);

            match wallet_key_db.create_record(wallet_key.clone()).await {
                Ok(Some(_)) => {
                    // Update the wallet record with the key ID
                    if let Some(wallet_db) = &self.wallet_db {
                        let wallet = wallet_db
                            .get_record_by_id(wallet_id)
                            .await
                            .map_err(|e| {
                                error!("Database error when fetching wallet for key update: {}", e);
                                AppError::DatabaseError(anyhow::anyhow!(e))
                            })?
                            .ok_or_else(|| {
                                AppError::NotFoundError(format!("Wallet with ID '{}' not found", wallet_id))
                            })?;

                        // Update the wallet with the key ID
                        let mut updated_wallet = wallet.clone();
                        updated_wallet.key_id = Some(key_id.clone());
                        updated_wallet.updated_at = chrono::Utc::now();

                        // Save the updated wallet
                        wallet_db.update_record(wallet_id, updated_wallet).await.map_err(|e| {
                            error!("Failed to update wallet with key ID: {}", e);
                            AppError::DatabaseError(anyhow::anyhow!(e))
                        })?;
                    }
                    
                    Ok(key_id)
                },
                Ok(None) => {
                    error!("Database did not return stored wallet key");
                    Err(AppError::DatabaseError(anyhow::anyhow!("Failed to store wallet key")))
                }
                Err(e) => {
                    error!("Failed to store wallet key in database: {}", e);
                    Err(AppError::DatabaseError(anyhow::anyhow!(format!("Failed to store wallet key: {}", e))))
                }
            }
        } else {
            error!("Wallet key database not available for storing key");
            Err(AppError::ServerError(anyhow::anyhow!("Wallet key database not available")))
        }
    }

    /// Get a wallet key by wallet ID
    pub async fn get_wallet_key_by_wallet_id(&self, wallet_id: &str) -> AppResult<WalletKey> {
        if let Some(wallet_db) = &self.wallet_db {
            // First get the wallet to find the key ID
            let wallet = wallet_db
                .get_record_by_id(wallet_id)
                .await
                .map_err(|e| {
                    error!("Database error when fetching wallet for key retrieval: {}", e);
                    AppError::DatabaseError(anyhow::anyhow!(e))
                })?
                .ok_or_else(|| {
                    AppError::NotFoundError(format!("Wallet with ID '{}' not found", wallet_id))
                })?;

            // Get the key ID from the wallet
            let key_id = wallet.key_id.clone().ok_or_else(|| {
                AppError::ValidationError(format!("Wallet '{}' has no associated key", wallet_id))
            })?;

            // Get the key from the keys table
            if let Some(wallet_key_db) = &self.wallet_key_db {
                let key = wallet_key_db
                    .get_record_by_id(&key_id)
                    .await
                    .map_err(|e| {
                        error!("Database error when fetching wallet key: {}", e);
                        AppError::DatabaseError(anyhow::anyhow!(format!("Failed to fetch wallet key: {}", e)))
                    })?
                    .ok_or_else(|| {
                        AppError::NotFoundError(format!("Wallet key with ID '{}' not found", key_id))
                    })?;

                Ok(key)
            } else {
                error!("Wallet key database not available for key retrieval");
                Err(AppError::ServerError(anyhow::anyhow!("Wallet key database not available")))
            }
        } else {
            error!("Wallet database not available for key retrieval");
            Err(AppError::ServerError(anyhow::anyhow!("Wallet database not available")))
        }
    }

    /// Get encrypted data from wallet ID for decryption
    pub async fn get_wallet_encrypted_data(&self, wallet_id: &str) -> AppResult<WalletEncryptedData> {
        let key = self.get_wallet_key_by_wallet_id(wallet_id).await?;
        Ok(Self::wallet_key_to_encrypted_data(&key))
    }

    /// Update a wallet key with new encrypted data (for PIN changes or master key rotation)
    pub async fn update_wallet_key(
        &self,
        wallet_id: &str,
        new_encrypted_data: &WalletEncryptedData
    ) -> AppResult<()> {
        // Get the current key first
        let current_key = self.get_wallet_key_by_wallet_id(wallet_id).await?;
        
        // Update the key with new encrypted data
        let mut updated_key = current_key.clone();
        updated_key.encrypted_private_key = new_encrypted_data.encrypted_private_key.clone();
        updated_key.encrypted_dek = new_encrypted_data.encrypted_dek.clone();
        updated_key.master_key_id = new_encrypted_data.master_key_identifier.clone();
        updated_key.dek_id = new_encrypted_data.dek_id.clone();
        updated_key.algorithm = new_encrypted_data.algorithm.clone();
        updated_key.pin_salt = new_encrypted_data.pin_salt.clone();
        updated_key.pin_iv = new_encrypted_data.pin_iv.clone();
        updated_key.dek_iv = new_encrypted_data.dek_iv.clone();
        updated_key.master_iv = new_encrypted_data.master_iv.clone();
        updated_key.updated_at = chrono::Utc::now();
        
        // Save the updated key
        if let Some(wallet_key_db) = &self.wallet_key_db {
            wallet_key_db
                .update_record(&current_key.id.id.to_string(), updated_key)
                .await
                .map_err(|e| {
                    error!("Failed to update wallet key: {}", e);
                    AppError::DatabaseError(anyhow::anyhow!(format!("Failed to update wallet key: {}", e)))
                })?;
            
            Ok(())
        } else {
            error!("Wallet key database not available for key update");
            Err(AppError::ServerError(anyhow::anyhow!("Wallet key database not available")))
        }
    }

    /// Rotate master key for a specific wallet
    pub async fn rotate_master_key(
        &self, 
        wallet_id: &str, 
        pin: &str, 
        new_encryption_service: &Arc<WalletEncryptionService>
    ) -> AppResult<()> {
        // 1. Get the current wallet key
        let key = self.get_wallet_key_by_wallet_id(wallet_id).await?;
        
        // 2. Convert to encrypted data format for decryption
        let encrypted_data = Self::wallet_key_to_encrypted_data(&key);
        
        // 3. Decrypt the private key using current encryption service
        let private_key = self.encryption_service
            .decrypt_private_key(&encrypted_data, pin)
            .await?;
            
        // 4. Re-encrypt with the new encryption service
        let new_encrypted_data = new_encryption_service
            .encrypt_private_key(&private_key, pin)
            .await?;
            
        // 5. Update the wallet key with the new encrypted data
        self.update_wallet_key(wallet_id, &new_encrypted_data).await?;
        
        info!("Successfully rotated master key for wallet {}", wallet_id);
        Ok(())
    }

    /// Rotate master key for all wallets (batch operation)
    pub async fn rotate_all_master_keys(
        &self,
        new_encryption_service: &Arc<WalletEncryptionService>,
        pin_provider: impl Fn(&str) -> AppResult<String>
    ) -> AppResult<(usize, Vec<String>)> {
        let mut successful = 0;
        let mut failed_wallets = Vec::new();
        
        // Get all wallet keys needing rotation (with old master key ID)
        if let Some(wallet_key_db) = &self.wallet_key_db {
            // Find keys with the old master key ID
            let old_master_key_id = &self.encryption_service.master_key_id;
            
            // Query for keys with the old master key ID
            // Note: In a real implementation, you'd use a more efficient query
            let old_keys = wallet_key_db
                .get_records_by_field("master_key_id", old_master_key_id.to_string())
                .await
                .map_err(|e| {
                    error!("Database error when fetching keys for rotation: {}", e);
                    AppError::DatabaseError(anyhow::anyhow!(format!("Failed to fetch keys for rotation: {}", e)))
                })?;
                
            info!("Found {} wallet keys to rotate", old_keys.len());
                
            // Process each key
            for key in old_keys {
                let wallet_id = &key.wallet_id;
                
                // Get PIN for this wallet
                match pin_provider(wallet_id) {
                    Ok(pin) => {
                        // Attempt to rotate this key
                        match self.rotate_master_key(wallet_id, &pin, new_encryption_service).await {
                            Ok(_) => {
                                successful += 1;
                            }
                            Err(e) => {
                                error!("Failed to rotate key for wallet {}: {}", wallet_id, e);
                                failed_wallets.push(wallet_id.clone());
                            }
                        }
                    }
                    Err(e) => {
                        error!("Failed to get PIN for wallet {}: {}", wallet_id, e);
                        failed_wallets.push(wallet_id.clone());
                    }
                }
            }
        } else {
            return Err(AppError::ServerError(anyhow::anyhow!("Wallet key database not available")));
        }
        
        info!("Master key rotation completed: {} successful, {} failed", successful, failed_wallets.len());
        Ok((successful, failed_wallets))
    }
}