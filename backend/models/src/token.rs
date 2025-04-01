use async_graphql::{SimpleObject, ID};
use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};

use crate::user::User;

// Token balance structure (simplified for the user service)
#[derive(SimpleObject, Clone, Serialize, Deserialize, Debug)]
pub struct TokenBalance {
    pub contract_address: String,
    pub balance: String,
}

// Token Profile model (simplified for the user service)
#[derive(SimpleObject, Clone, Serialize, Deserialize, Debug)]
pub struct TokenProfile {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<ID>,
    pub user_id: ID,
    pub native_balance: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// Authentication response type
#[derive(SimpleObject)]
pub struct AuthPayload {
    pub token: String,
    pub user: User,
}