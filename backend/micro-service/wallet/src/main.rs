use anyhow::Context;
use app_config::AppConfig;
use app_database::{db_connect::initialize_wallet_db, DB_ARC};
use app_database::service::DbService;
use app_error::AppError;
use app_models::wallet::Wallet;
use std::sync::Arc;
use tokio::net::TcpListener;
use tracing::{error, info, Level};
use tracing_subscriber::{layer::SubscriberExt, FmtSubscriber};

#[tokio::main]
async fn main() -> Result<(), AppError> {
    // Load configuration from JSON file
    let config = AppConfig::load().context("Failed to load application configuration")?;

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

    let subscriber = FmtSubscriber::builder().with_max_level(log_level).finish();

    let subscriber = subscriber.with(sentry_tracing::layer());
    tracing::subscriber::set_global_default(subscriber).expect("Failed to set tracing subscriber");

    info!(
        "Starting wallet service in {} environment at {}",
        config.environment,
        chrono::Utc::now()
    );

    let _wallet_db_arc = DB_ARC
    .get_or_init(|| async {
        initialize_wallet_db().await.unwrap_or_else(|e| {
            error!("Wallet database initialization failed: {}", e);
            panic!("Wallet database initialization failed");
        })
    })
    .await;

    // // Create DB service for Wallet model
    // let _wallet_db = Arc::new(DbService::<Wallet>::new(&wallet_db_arc, "wallets"));

    info!("Wallet database connection established");

    // TODO: Initialize wallet-specific services and routes
    
    // Bind server to address and start it
    // Using a different port than the user service
    let wallet_port = config.server.port + 1;  // Example: use a different port
    let address = format!("{}:{}", config.server.host, wallet_port);
    let listener = TcpListener::bind(&address)
        .await
        .context(format!("Failed to bind to address: {}", address))?;

    info!("Wallet service API available at: http://{}", address);

    // TODO: Create and start wallet-specific server 
    // For now, just keep the service alive
    info!("Wallet service starting on {}", address);
    
    // Example placeholder for future wallet-specific server implementation
    let app = axum::Router::new()
        .route("/", axum::routing::get(|| async { "Wallet Service Healthy" }));
    
    axum::serve(listener, app).await.context("Server error")?;

    Ok(())
}