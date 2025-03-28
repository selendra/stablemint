use chrono::{DateTime, Utc}; // For Date handling

#[derive(Debug)]
pub struct User {
    pub name: String,
    pub username: String, // Unique
    pub email: String,    // Unique
    pub password: String,
    pub address: String,     // Unique
    pub private_key: String, // Unique
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
