use anyhow::Context;
use app_config::AppConfig;
use app_database::{
    db_connect::{initialize_user_db, initialize_wallet_db}, service::DbService, USER_DB_ARC, WALLET_DB_ARC
};
use app_error::AppError;
use app_middleware::{JwtService, limits::rate_limiter::create_redis_api_rate_limiter};
use app_models::{user::User, wallet::Wallet};
use app_utils::secrets::SecretsClient;
use app_utils::crypto::WalletEncryptionService;
use micro_wallet::{routes, schema::create_schema, service::WalletService};
use std::{collections::HashMap, sync::Arc};
use tokio::net::TcpListener;
use tracing::{Level, error, info};
use tracing_subscriber::{FmtSubscriber, layer::SubscriberExt};

#[tokio::main]
async fn main() -> Result<(), AppError> {
    // Load the application configuration from JSON file
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

    let wallet_db_arc = WALLET_DB_ARC
        .get_or_init(|| async {
            initialize_wallet_db().await.unwrap_or_else(|e| {
                error!("Wallet database initialization failed: {}", e);
                panic!("Wallet database initialization failed");
            })
        })
        .await;
    let wallet_db = Arc::new(DbService::<Wallet>::new(&wallet_db_arc, "wallets"));

    let user_db_arc = USER_DB_ARC
        .get_or_init(|| async {
            initialize_user_db().await.unwrap_or_else(|e| {
                error!("Database initialization failed: {}", e);
                panic!("Database initialization failed");
            })
        })
        .await;
    let user_db = Arc::new(DbService::<User>::new(&user_db_arc, "users"));

    // Configure path-specific rate limits from our config file
    let mut path_limits = HashMap::new();

    // Convert path-specific limits from the config
    for (path, limit) in &config.security.rate_limiting.paths {
        path_limits.insert(path.clone(), *limit);
    }

    // Initialize Redis configuration
    let redis_config = config.redis.clone().ok_or_else(|| {
        AppError::ConfigError(anyhow::anyhow!(
            "Redis configuration is required but not provided"
        ))
    })?;

    info!("Initializing Redis-based distributed rate limiting");

    // Create API rate limiter with Redis backend
    let api_rate_limiter =
        Arc::new(create_redis_api_rate_limiter(&redis_config.url, Some(path_limits)).await?);

    // Create JWT service for token validation
    let jwt_service = Arc::new(JwtService::new(
        config.security.jwt.secret.as_bytes(),
        config.security.jwt.expiry_hours,
    ));

    // Get HCP Secrets configuration
    let hcp_secrets = config.hcp_secrets.clone().ok_or_else(|| {
        AppError::ConfigError(anyhow::anyhow!(
            "HCP Secrets configuration is required but not provided"
        ))
    })?;

    // Create HCP Secrets client
    info!("Initializing HCP Secrets client...");
    let secrets_client = Arc::new(SecretsClient::new(
        &hcp_secrets.base_url,
        &hcp_secrets.org_id,
        &hcp_secrets.project_id,
        &hcp_secrets.app_name,
        &hcp_secrets.client_id,
        &hcp_secrets.client_secret,
    ));

    // Initialize HCP Secrets client
    match secrets_client.initialize().await {
        Ok(_) => info!("Successfully authenticated with HCP Secrets"),
        Err(e) => {
            error!("Failed to authenticate with HCP Secrets: {}", e);
            return Err(AppError::ConfigError(anyhow::anyhow!(
                "HCP Secrets authentication failed: {}", e
            )));
        }
    }

    // Create encryption service
    let encryption_service = Arc::new(WalletEncryptionService::new(Arc::clone(&secrets_client)));

    // Initialize encryption service
    match encryption_service.initialize().await {
        Ok(_) => info!("Successfully initialized encryption service"),
        Err(e) => {
            error!("Failed to initialize encryption service: {}", e);
            return Err(AppError::ConfigError(anyhow::anyhow!(
                "Encryption service initialization failed: {}", e
            )));
        }
    }

    // Create wallet service
    let wallet_service = WalletService::new(encryption_service)
        .with_wallet_db(wallet_db)
        .with_user_db(user_db);
    
    let wallet_service = Arc::new(wallet_service);

    // Create GraphQL schema
    let schema = create_schema();

    // Configure application routes
    let app = routes::create_routes(schema, wallet_service, api_rate_limiter, jwt_service);

    // Bind server to address and start it
    // Use a different port than the user service
    let wallet_port = config.server.port + 1; // Use a different port
    let address = format!("{}:{}", config.server.host, wallet_port);
    let listener = TcpListener::bind(&address)
        .await
        .context(format!("Failed to bind to address: {}", address))?;

    info!(
        "Wallet service GraphQL playground available at: http://{}/graphql",
        address
    );

    // Start server with graceful error handling
    info!("Wallet service starting on {}", address);
    axum::serve(listener, app).await.context("Server error")?;

    Ok(())
}