use anyhow::Context;
use backend::{
    config::Server,
    error::{AppError, AppErrorExt},
    routes,
    schema::create_schema,
};
use tokio::net::TcpListener;
use tracing::{Level, info};
use tracing_subscriber::FmtSubscriber;

#[tokio::main]
async fn main() -> Result<(), AppError> {
    // Initialize the logger
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .finish();
    tracing::subscriber::set_global_default(subscriber).expect("Failed to set tracing subscriber");

    info!("Starting application");

    // Load configuration with better error handling
    let config = Server::from_env()
        .context("Failed to load server configuration")
        .config_err()?;

    info!("Configuration loaded successfully");

    // Create GraphQL schema
    let schema = create_schema();
    info!("GraphQL schema created");

    // Build the application with routes
    let app = routes::create_routes(schema);
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
