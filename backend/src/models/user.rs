use async_graphql::{InputObject, SimpleObject};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct User {
    pub id: Uuid,
    pub name: String,
    pub username: String,
    pub email: String,
    pub password: String,
    pub address: String,
    pub private_key: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, SimpleObject, Serialize, Deserialize)]
pub struct UserProfile {
    pub id: String,
    pub name: String,
    pub username: String,
    pub email: String,
    pub address: String,
    pub created_at: DateTime<Utc>,
}

// Convert User to UserProfile (hiding sensitive data)
impl From<User> for UserProfile {
    fn from(user: User) -> Self {
        Self {
            id: user.id.to_string(),
            name: user.name,
            username: user.username,
            email: user.email,
            address: user.address,
            created_at: user.created_at,
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
