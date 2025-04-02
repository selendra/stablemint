use async_graphql::{EmptySubscription, Schema};
use graphql::{mutation::MutationRoot, query::QueryRoot};
use stablemint_authentication::{JwtAuth, JwtConfig};
use stablemint_surrealdb::conn::credentials::{ConnectionManager, DatabaseCredentials, SecureDatabaseConfig};

pub mod graphql;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    // Load environment variables from .env file
    dotenv::dotenv().ok();

    // Set up JWT
    let jwt_config = JwtConfig::from_env()?;
    let jwt_auth = JwtAuth::new(jwt_config);

     // Database connection
     let db_config = SecureDatabaseConfig::new(
        std::env::var("DB_ENDPOINT").unwrap_or_else(|_| "memory".to_string()),
        DatabaseCredentials::from_env("DB_USERNAME", "DB_PASSWORD")?,
        std::env::var("DB_NAMESPACE").unwrap_or_else(|_| "test".to_string()),
        std::env::var("DB_DATABASE").unwrap_or_else(|_| "test".to_string()),
    );

    let mut conn_manager = ConnectionManager::new(db_config);
    let db = conn_manager.get_connection().await?;

     // Set up GraphQL schema
     let schema = Schema::build(QueryRoot::default(), MutationRoot::default(), EmptySubscription)
     .data(db.clone())
     .finish();
    
    Ok(())
}
