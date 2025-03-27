use anyhow::{Context, Result};
use dotenv::dotenv;
use serde::{Deserialize, Serialize};
use std::{env, sync::Arc};
use surrealdb::sql::Thing;
use surrealdb::{Surreal, engine::any::Any, opt::auth::Root};

pub type Database = Arc<Surreal<Any>>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    pub endpoint: String,
    pub username: String,
    pub password: String,
    pub namespace: String,
    pub database: String,
}

impl DatabaseConfig {
    pub fn from_env() -> Result<Self> {
        // Load .env file only once per process
        dotenv().ok();

        Ok(Self {
            endpoint: env::var("SURREALDB_ENDPOINT")
                .unwrap_or_else(|_| "ws://localhost:8000".to_string()),
            username: env::var("SURREALDB_USERNAME").unwrap_or_else(|_| "root".to_string()),
            password: env::var("SURREALDB_PASSWORD").unwrap_or_else(|_| "root".to_string()),
            namespace: env::var("SURREALDB_NAMESPACE").unwrap_or_else(|_| "selendraDb".to_string()),
            database: env::var("SURREALDB_DATABASE").unwrap_or_else(|_| "cryptoBank".to_string()),
        })
    }
}

pub async fn create_db_pool() -> Result<Database> {
    let config = DatabaseConfig::from_env().context("Failed to load database configuration")?;

    tracing::debug!("Connecting to SurrealDB: {}", config.endpoint);

    // Connect to the database
    let db = surrealdb::engine::any::connect(&config.endpoint)
        .await
        .context("Failed to connect to SurrealDB")?;

    // Authenticate to the database
    db.signin(Root {
        username: &config.username,
        password: &config.password,
    })
    .await
    .context("Failed to authenticate to SurrealDB")?;

    // Use a single operation to select namespace and database
    db.use_ns(&config.namespace)
        .use_db(&config.database)
        .await
        .context("Failed to select namespace and database")?;

    tracing::info!("Successfully connected to SurrealDB at {}", config.endpoint);

    Ok(Arc::new(db))
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: Thing,
    pub name: String,
    pub email: String,
}

// Create a new user
pub async fn create_user(db: &Database, user: User) -> Result<Option<User>> {
    // Fixed: Pass the user data to create
    let _user = &user;
    let created_user: Option<User> = db
        .create("user")
        .content(_user.clone()) // Add this line to pass the user data
        .await
        .context("Failed to create user")?; // Fixed: Use ? instead of unwrap

    Ok(created_user)
}

// Read a user by string id
pub async fn read_user_by_id(db: &Database, user_id: &str) -> Result<Option<User>> {
    let user: Option<User> = db
        .select(("user", user_id))
        .await
        .context("Failed to read user by ID")?;

    Ok(user)
}

#[tokio::main]
async fn main() -> Result<()> {
    let db = create_db_pool().await?;

    // // Create a new user
    // let new_user = User {
    //     id: Thing::from(("user", "13")),
    //     name: "John Doe".to_string(),
    //     email: "john.doe@example.com".to_string(),
    // };
    // let created_user = create_user(&db, new_user).await?;
    // println!("Created User: {:?}", created_user);

    // Read user by string id
    let user_id_str = "13";
    let read_result_by_str = read_user_by_id(&db, user_id_str).await?;
    println!("Read User by string ID: {:?}", read_result_by_str);

    Ok(())
}
