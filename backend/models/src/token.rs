use std::str;

use async_graphql::{SimpleObject, ID};
use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};

// Token balance structure (simplified for the user service)
#[derive(SimpleObject, Clone, Serialize, Deserialize, Debug)]
pub struct Token {
    pub contract_address: String,
    pub balance: String,
}

// Token Profile model (simplified for the user service)
#[derive(SimpleObject, Clone, Serialize, Deserialize, Debug)]
pub struct TokenProfile {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<ID>,
    pub user_id: ID,
    pub token: Vec<Token>,
    pub native_balance: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

pub enum TokenType {
    ERC20,
    ERC721 
}

pub struct Contract {
    pub user_id: ID,
    pub contract_address: String,
    pub created_at: String,
    pub contract_type: TokenType,
}