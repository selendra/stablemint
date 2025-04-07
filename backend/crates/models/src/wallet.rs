use serde::{Deserialize, Serialize};
use surrealdb::sql::Thing;
use chrono::{DateTime, Utc};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Wallet {
    #[serde(default = "Wallet::generate_id")]
    pub id: Thing,
    pub user_email: String,
    pub address: String,
    pub private_key: String,  // Implement encryption for production
    pub mnemonic: String,     // Implement encryption for production
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
    pub fn new(
        user_email: String,
        address: String,
        private_key: String,
        mnemonic: String,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: Self::generate_id(),
            user_email,
            address,
            private_key, 
            mnemonic,
            created_at: now,
            updated_at: now,
        }
    }
}

// For API responses (without sensitive data)
#[derive(Debug, Serialize, Deserialize, Clone)]
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