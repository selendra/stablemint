use serde::{Deserialize, Serialize};
use surrealdb::sql::Thing;
use chrono::{DateTime, Utc};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Wallet {
    pub id: Thing,
    pub user_email: String,
    pub address: String,
    pub private_key: String,  // todo! implement encryption later
    pub mnemonic: String,  // todo! implement encryption later
    #[serde(default = "Utc::now")]
    pub created_at: DateTime<Utc>,
    #[serde(default = "Utc::now")]
    pub updated_at: DateTime<Utc>,
}