// backend/micro-service/wallet/src/routes.rs
use crate::{
    handlers::graphql::{graphql_handler, graphql_playground, health_check},
    middleware::wallet_owner_middleware,
    schema::ApiSchema,
    service::WalletService,
};
use std::{sync::Arc, time::Duration};
use tower::ServiceBuilder;
use tower_http::{
    compression::CompressionLayer,
    cors::CorsLayer,
    timeout::TimeoutLayer,
    trace::TraceLayer,
};

use axum::{
    Router,
    body::Body,
    extract::{Extension, Request, State},
    middleware::Next,
    routing::get,
};
use tower_http::limit::RequestBodyLimitLayer;

use app_config::AppConfig;
use app_error::middleware_handling::error_handling_middleware;
use app_middleware::{
    Claims, JwtService,
    api_middleware::{
        api_rate_limit_middleware, jwt_auth_middleware, logging_middleware,
        security_headers_middleware,
    },
    limits::rate_limiter::RedisApiRateLimiter,
};

pub fn create_routes(
    schema: ApiSchema,
    wallet_service: Arc<WalletService>,
    api_rate_limiter: Arc<RedisApiRateLimiter>,
    jwt_service: Arc<JwtService>,
) -> Router {
    // Load configuration
    let config = AppConfig::load().unwrap_or_default();
    
    // Get body limit and CORS settings from config
    let body_limit = config.server.body_limit;
    let cors_config = &config.security.cors;
    
    // Configure CORS with settings from config
    let cors = CorsLayer::new()
        // If allowed_origins contains "*", use Any, otherwise use exact list
        .allow_origin(
            if cors_config.allowed_origins.contains(&"*".to_string()) {
                tower_http::cors::AllowOrigin::any()
            } else {
                tower_http::cors::AllowOrigin::list(
                    cors_config.allowed_origins.iter()
                        .filter_map(|origin| origin.parse().ok())
                        .collect::<Vec<_>>()
                )
            }
        )
        // Convert allowed methods from strings to HTTP methods
        .allow_methods(
            cors_config.allowed_methods.iter()
                .filter_map(|method| method.parse().ok())
                .collect::<Vec<_>>()
        )
        // Convert allowed headers from strings to HTTP header names
        .allow_headers(
            cors_config.allowed_headers.iter()
                .filter_map(|header| header.parse().ok())
                .collect::<Vec<_>>()
        );

    // Define global middleware stack WITHOUT the body limit
    let middleware_stack = ServiceBuilder::new()
        .layer(TraceLayer::new_for_http())
        .layer(CompressionLayer::new())
        .layer(TimeoutLayer::new(Duration::from_secs(30)))
        .layer(cors);

    // Build router with optimized middleware stack
    let app = Router::new()
        .route("/", get(graphql_playground))
        .route("/health", get(health_check))
        .route("/graphql", get(graphql_playground).post(graphql_handler));

    // Add Extensions
    let app = app
        .layer(Extension(schema))
        .layer(Extension(Arc::clone(&wallet_service)))
        .layer(Extension(jwt_service.clone()))
        .layer(Extension(Arc::clone(&api_rate_limiter)));

    // Apply middleware in order
    let app = app
        .layer(axum::middleware::from_fn(error_handling_middleware))
        .layer(RequestBodyLimitLayer::new(body_limit));  // Use body limit from config

    // Apply custom middleware stacks
    let app = app
        .layer(axum::middleware::from_fn(logging_middleware))
        .layer(axum::middleware::from_fn(security_headers_middleware))
        .layer(axum::middleware::from_fn_with_state(
            wallet_service.clone(),
            |State(state): State<Arc<WalletService>>, req: Request<Body>, next: Next| async move {
                wallet_owner_middleware(
                    req.extensions().get::<Claims>().cloned(),
                    State(state),
                    req,
                    next,
                )
                .await
            },
        ));

    // Use with_state method instead of direct middleware application
    let app = app
        .layer(axum::middleware::from_fn_with_state(
            api_rate_limiter.clone(),
            api_rate_limit_middleware,
        ))
        .layer(axum::middleware::from_fn_with_state(
            jwt_service,
            jwt_auth_middleware,
        ));

    // Apply global middleware stack
    app.layer(middleware_stack)
}
