// backend/crates/models/src/user.rs
use async_graphql::{InputObject, SimpleObject};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use surrealdb::sql::Thing;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct User {
    #[serde(default = "User::generate_id")]
    pub id: Thing,
    pub name: String,
    pub username: String,
    pub email: String,
    pub password: String,
    #[serde(default = "Utc::now")]
    pub created_at: DateTime<Utc>,
    #[serde(default = "Utc::now")]
    pub updated_at: DateTime<Utc>,
    // Add optional wallet reference
    #[serde(skip_serializing_if = "Option::is_none")]
    pub wallet_id: Option<String>,
}

impl User {
    // Helper to generate a new ID
    fn generate_id() -> Thing {
        Thing::from(("users".to_string(), Uuid::new_v4().to_string()))
    }

    // Create a new user with default values for fields that aren't provided
    pub fn new(name: String, username: String, email: String, password: String) -> Self {
        let now = Utc::now();
        Self {
            id: Self::generate_id(),
            name,
            username,
            email,
            password,
            created_at: now,
            updated_at: now,
            wallet_id: None,
        }
    }

    // Associate a wallet with this user
    pub fn set_wallet(&mut self, wallet_id: String) {
        self.wallet_id = Some(wallet_id);
        self.updated_at = Utc::now();
    }
}

#[derive(Debug, SimpleObject, Serialize, Deserialize)]
pub struct UserProfile {
    pub id: String,
    pub name: String,
    pub username: String,
    pub email: String,
    pub created_at: DateTime<Utc>,
    pub wallet_id: Option<String>,
}

// Convert User to UserProfile (hiding sensitive data)
impl From<User> for UserProfile {
    fn from(user: User) -> Self {
        Self {
            id: user.id.id.to_string(),
            name: user.name,
            username: user.username,
            email: user.email,
            created_at: user.created_at,
            wallet_id: user.wallet_id,
        }
    }
}

#[derive(InputObject, Debug, Deserialize)]
pub struct RegisterInput {
    pub name: String,
    pub username: String,
    pub email: String,
    pub password: String,
}

#[derive(InputObject, Debug, Deserialize)]
pub struct LoginInput {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Serialize, SimpleObject)]
pub struct AuthResponse {
    pub token: String,
    pub user: UserProfile,
}
