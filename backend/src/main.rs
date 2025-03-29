use anyhow::Context;
use backend::{
    config::Server,
    database::{db_connect::initialize_db, operation::DbService},
    errors::{AppError, AppErrorExt},
    handlers::auth::AuthService,
    models::user::User,
    routes,
    schema::create_schema,
    types::DB_ARC,
};
use std::sync::Arc;

extern crate lazy_static;
use tokio::net::TcpListener;
use tracing::{Level, error, info, warn};
use tracing_subscriber::FmtSubscriber;

#[tokio::main]
async fn main() -> Result<(), AppError> {
    let _guard = sentry::init((
        "",
        sentry::ClientOptions {
            release: sentry::release_name!(),
            ..Default::default()
        },
    ));

    // Initialize the logger
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .finish();
    tracing::subscriber::set_global_default(subscriber).expect("Failed to set tracing subscriber");

    info!("Starting application at {}", chrono::Utc::now());

    // Load configuration with better error handling
    let config = Server::from_env()
        .context("Failed to load server configuration")
        .config_err()?;

    info!("Configuration loaded successfully");

    // Initialize the database and store it in OnceCell
    let db_arc = DB_ARC
        .get_or_init(|| async {
            initialize_db().await.unwrap_or_else(|e| {
                error!("Database initialization failed: {}", e);
                panic!("Database initialization failed");
            })
        })
        .await;

    // Create DB service for users
    let user_db = Arc::new(DbService::<User>::new(db_arc, "users"));

    info!("Database connection established");

    // Set up auth service with database
    let jwt_secret = std::env::var("JWT_SECRET")
        .unwrap_or_else(|_| {
            warn!("JWT_SECRET not set, using fallback secret (not secure for production)");
            "your_fallback_secret_key_for_development_only".to_string()
        })
        .into_bytes();

    let auth_service = Arc::new(AuthService::new(&jwt_secret).with_db(user_db));

    info!("Authentication service initialized with database");

    // Create GraphQL schema
    let schema = create_schema(Some(Arc::clone(&auth_service)));
    info!("GraphQL schema created");

    // Build the application with routes
    let app = routes::create_routes(schema, auth_service);
    info!("Application routes configured");

    // Set up server address
    let address = format!("{}:{}", config.address, config.port);
    let listener = TcpListener::bind(&address)
        .await
        .context(format!("Failed to bind to address: {}", address))
        .server_err()?;

    info!("GraphQL playground available at: http://{}", address);

    // Start the server with graceful error handling
    info!("Server starting");
    axum::serve(listener, app)
        .await
        .context("Server error")
        .server_err()?;

    Ok(())
}
