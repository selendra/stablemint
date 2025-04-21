use async_graphql::SimpleObject;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use surrealdb::sql::Thing;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Wallet {
    #[serde(default = "Wallet::generate_id")]
    pub id: Thing,
    pub user_email: String,
    pub address: String,
    // We'll replace the private_key field with a reference to the WalletKey
    #[serde(skip_serializing_if = "Option::is_none")]
    pub key_id: Option<String>,  // Reference to the WalletKey record
    #[serde(default = "Utc::now")]
    pub created_at: DateTime<Utc>,
    #[serde(default = "Utc::now")]
    pub updated_at: DateTime<Utc>,
}

impl Wallet {
    // Helper to generate a new ID
    pub fn generate_id() -> Thing {
        Thing::from(("wallets".to_string(), Uuid::new_v4().to_string()))
    }

    // Create a new wallet with all required fields
    pub fn new(user_email: String, address: String) -> Self {
        let now = Utc::now();
        Self {
            id: Self::generate_id(),
            user_email,
            address,
            key_id: None,  // Will be set after key is created
            created_at: now,
            updated_at: now,
        }
    }
    
    // Set the key ID
    pub fn with_key_id(mut self, key_id: String) -> Self {
        self.key_id = Some(key_id);
        self
    }
}

// For API responses (without sensitive data)
#[derive(Debug, SimpleObject, Serialize, Deserialize, Clone)]
pub struct WalletInfo {
    pub id: String,
    pub user_email: String,
    pub address: String,
    pub created_at: DateTime<Utc>,
}

impl From<Wallet> for WalletInfo {
    fn from(wallet: Wallet) -> Self {
        Self {
            id: wallet.id.id.to_string(),
            user_email: wallet.user_email,
            address: wallet.address,
            created_at: wallet.created_at,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct WalletKey {
    #[serde(default = "WalletKey::generate_id")]
    pub id: Thing,
    pub wallet_id: String,
    pub encrypted_private_key: String, // Hex-encoded AES-GCM encrypted private key (encrypted with DEK)
    pub encrypted_dek: String,         // Hex-encoded AES-GCM encrypted DEK (encrypted with master key)  
    pub master_key_id: String,         // Identifier for the master key used
    pub dek_id: String,                // ID for the DEK (used for caching)
    pub algorithm: String,             // Encryption algorithm used (e.g., "AES-256-GCM")
    pub pin_salt: String,              // Hex-encoded salt for PIN key derivation
    pub pin_iv: String,                // Hex-encoded IV for PIN encryption
    pub dek_iv: String,                // Hex-encoded IV for DEK encryption
    pub master_iv: String,             // Hex-encoded IV for master key encryption
    #[serde(default = "Utc::now")]
    pub created_at: DateTime<Utc>,
    #[serde(default = "Utc::now")]
    pub updated_at: DateTime<Utc>,
}

impl WalletKey {
    // Helper to generate a new ID
    pub fn generate_id() -> Thing {
        Thing::from(("wallet_keys".to_string(), Uuid::new_v4().to_string()))
    }

    // Create a new wallet key entry
    pub fn new(
        wallet_id: String,
        encrypted_private_key: String,
        encrypted_dek: String,
        master_key_id: String,
        dek_id: String,
        algorithm: String,
        pin_salt: String,
        pin_iv: String,
        dek_iv: String,
        master_iv: String,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: Self::generate_id(),
            wallet_id,
            encrypted_private_key,
            encrypted_dek,
            master_key_id,
            dek_id,
            algorithm,
            pin_salt,
            pin_iv,
            dek_iv,
            master_iv,
            created_at: now,
            updated_at: now,
        }
    }
}