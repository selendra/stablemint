use anyhow::{Context, Result};
use micro_user::{handlers::auth::AuthService, routes, schema::create_schema};
use std::sync::Arc;
use tokio::net::TcpListener;
use tracing::{Level, error, info, warn};
use tracing_subscriber::{FmtSubscriber, layer::SubscriberExt};

use app_config::{SentryConfig, Server};
use app_database::{DB_ARC, db_connect::initialize_db, service::DbService};
use app_error::AppError;
use app_models::user::User;

#[tokio::main]
async fn main() -> Result<(), AppError> {
    // Load and initialize sentry
    let sentry_config = SentryConfig::from_env().context("Failed to load sentry configuration")?;
    let _guard = sentry::init((
        sentry_config.sentry_dsn,
        sentry::ClientOptions {
            release: sentry::release_name!(),
            ..Default::default()
        },
    ));

    // Initialize the logger
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .finish();

    let subscriber = subscriber.with(sentry_tracing::layer());
    tracing::subscriber::set_global_default(subscriber).expect("Failed to set tracing subscriber");

    info!("Starting application at {}", chrono::Utc::now());

    // Load server configuration
    let config = Server::from_env().context("Failed to load server configuration")?;

    // Initialize the database connection with the new connection pool
    let db_arc = DB_ARC
        .get_or_init(|| async {
            initialize_db().await.unwrap_or_else(|e| {
                error!("Database initialization failed: {}", e);
                panic!("Database initialization failed");
            })
        })
        .await;

    let user_db = Arc::new(DbService::<User>::new(db_arc, "users"));

    // Setup authentication service
    let jwt_secret = std::env::var("JWT_SECRET")
        .unwrap_or_else(|_| {
            warn!("JWT_SECRET not set, using fallback secret (not secure for production)");
            "your_fallback_secret_key_for_development_only".to_string()
        })
        .into_bytes();

    let auth_service = Arc::new(AuthService::new(&jwt_secret).with_db(user_db));

    // Create GraphQL schema
    let schema = create_schema();

    // Configure application routes
    let app = routes::create_routes(schema, auth_service);

    // Bind server to address and start it
    let address = format!("{}:{}", config.address, config.port);
    let listener = TcpListener::bind(&address)
        .await
        .context(format!("Failed to bind to address: {}", address))?;

    info!("GraphQL playground available at: http://{}", address);

    // Start server with graceful error handling
    info!("Server starting with connection pool");
    axum::serve(listener, app).await.context("Server error")?;

    Ok(())
}
