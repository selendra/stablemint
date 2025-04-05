use anyhow::Context;
use app_middleware::limits::rate_limiter::RateLimiter;
use micro_user::{routes, service::AuthService};
use std::{sync::Arc, time::Duration};
use tokio::net::TcpListener;
use tracing::{Level, error, info};
use tracing_subscriber::{FmtSubscriber, layer::SubscriberExt};

use app_config::AppConfig;
use app_database::{DB_ARC, db_connect::initialize_db, service::DbService};
use app_error::AppError;
use app_models::user::User;
use micro_user::schema::create_schema;

#[tokio::main]
async fn main() -> Result<(), AppError> {
    // Load the application configuration from JSON file
    let config = AppConfig::load()
        .context("Failed to load application configuration")?;
    
    // Initialize Sentry with configuration from JSON
    let _guard = if !config.monitoring.sentry.dsn.is_empty() {
        info!("Initializing Sentry with DSN");
        Some(sentry::init((
            config.monitoring.sentry.dsn.clone(),
            sentry::ClientOptions {
                release: sentry::release_name!(),
                sample_rate: config.monitoring.sentry.sample_rate,
                traces_sample_rate: config.monitoring.sentry.traces_sample_rate,
                environment: Some(config.monitoring.sentry.environment.clone().into()),
                ..Default::default()
            },
        )))
    } else {
        info!("Sentry DSN not configured, skipping Sentry initialization");
        None
    };

    // Initialize the logger based on config
    let log_level = match config.monitoring.logging.level.to_lowercase().as_str() {
        "trace" => Level::TRACE,
        "debug" => Level::DEBUG,
        "info" => Level::INFO,
        "warn" => Level::WARN,
        "error" => Level::ERROR,
        _ => Level::INFO,
    };
    
    let subscriber = FmtSubscriber::builder()
        .with_max_level(log_level)
        .finish();

    let subscriber = subscriber.with(sentry_tracing::layer());
    tracing::subscriber::set_global_default(subscriber)
        .expect("Failed to set tracing subscriber");

    info!("Starting application in {} environment at {}", 
          config.environment, chrono::Utc::now());

    // Initialize the database connection with our config
    let db_arc = DB_ARC
        .get_or_init(|| async {
            initialize_db().await.unwrap_or_else(|e| {
                error!("Database initialization failed: {}", e);
                panic!("Database initialization failed");
            })
        })
        .await;

    let user_db = Arc::new(DbService::<User>::new(db_arc, "users"));
    
    // Configure rate limiting from our config file
    // Create login rate limiter with configured settings from JSON
    let login_limiter = Arc::new(RateLimiter::new(
        app_middleware::limits::rate_limiter::RateLimitConfig {
            max_attempts: config.security.rate_limiting.login.max_attempts,
            window_duration: Duration::from_secs(
                config.security.rate_limiting.login.window_duration
            ),
            block_duration: config.security.rate_limiting.login.block_duration
                .map(|seconds| Duration::from_secs(seconds)),
            message_template: "Account protection: Too many login attempts.".into(),
        }
    ));

    // Create auth service with JWT config from our config file
    let auth_service = Arc::new(
        AuthService::new(
            config.security.jwt.secret.as_bytes(),
            config.security.jwt.expiry_hours
        )
        .with_db(user_db)
        .with_rate_limiter(login_limiter)
    );

    // Create GraphQL schema
    let schema = create_schema();

    // Configure application routes
    let app = routes::create_routes(schema, auth_service);

    // Bind server to address and start it
    let address = format!("{}:{}", config.server.host, config.server.port);
    let listener = TcpListener::bind(&address)
        .await
        .context(format!("Failed to bind to address: {}", address))?;

    info!("GraphQL playground available at: http://{}/graphql", address);

    // Start server with graceful error handling
    info!("Server starting on {}", address);
    axum::serve(listener, app)
        .await
        .context("Server error")?;

    Ok(())
}
