use async_graphql::{Enum, ID, InputObject, SimpleObject};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use surrealdb::sql::Thing;

// Role enum for user permissions
#[derive(Enum, Copy, Clone, Eq, PartialEq, Serialize, Deserialize, Debug)]
pub enum UserRole {
    Admin,
    User,
    Council,
}

// User model
#[derive(SimpleObject, Clone, Serialize, Deserialize, Debug)]
pub struct User {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<ID>,
    pub username: String,
    #[serde(skip_serializing)] // Don't return password in API responses
    pub password: String,
    pub email: String,
    pub address: String,
    #[graphql(skip)] // Don't expose private key in GraphQL
    pub private_key: String,
    pub role: UserRole,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// Input objects for mutations
#[derive(InputObject)]
pub struct CreateUserInput {
    pub username: String,
    pub password: String,
    pub email: String,
    pub address: String,
    pub role: UserRole,
}

#[derive(InputObject)]
pub struct UpdateUserInput {
    pub username: Option<String>,
    pub email: Option<String>,
    pub address: Option<String>,
    pub role: Option<UserRole>,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct DBUser {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<Thing>,
    pub username: String,
    pub password: String,
    pub email: String,
    pub address: String,
    pub private_key: String,
    pub role: UserRole,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// Database conversion helpers
impl User {
    pub fn from_db(db_user: DBUser) -> Self {
        User {
            id: db_user.id.map(|thing| ID(thing.id.to_string())),
            username: db_user.username,
            password: db_user.password,
            email: db_user.email,
            address: db_user.address,
            private_key: db_user.private_key,
            role: db_user.role,
            created_at: db_user.created_at,
            updated_at: db_user.updated_at,
        }
    }
}
