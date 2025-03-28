use crate::{
    handlers::{
        auth::AuthService,
        graphql::{graphql_handler, graphql_playground, health_check},
    },
    middleware::{auth::jwt::JwtService, error::error_handling_middleware},
    schema::ApiSchema,
};
use axum::{Router, extract::Extension, middleware, routing::get};
use std::{sync::Arc, time::Duration};
use tower::ServiceBuilder;
use tower_http::{
    compression::CompressionLayer,
    cors::{Any, CorsLayer},
    timeout::TimeoutLayer,
    trace::TraceLayer,
};

// Create application routes with middleware
pub fn create_routes(schema: ApiSchema) -> Router {
    // Create auth services with proper error handling
    let jwt_secret = std::env::var("JWT_SECRET")
        .unwrap_or_else(|_| {
            tracing::warn!("JWT_SECRET not set, using fallback secret (not secure for production)");
            "your_fallback_secret_key_for_development_only".to_string()
        })
        .into_bytes();

    let jwt_service = Arc::new(JwtService::new(&jwt_secret));
    let auth_service = Arc::new(AuthService::new(&jwt_secret));

    tracing::info!("Authentication services initialized");

    // Define global middleware stack
    let middleware_stack = ServiceBuilder::new()
        .layer(TraceLayer::new_for_http())
        .layer(CompressionLayer::new())
        .layer(TimeoutLayer::new(Duration::from_secs(30)))
        .layer(
            CorsLayer::new()
                .allow_origin(Any)
                .allow_headers([
                    axum::http::header::CONTENT_TYPE,
                    axum::http::header::AUTHORIZATION,
                ])
                .allow_methods([
                    axum::http::Method::GET,
                    axum::http::Method::POST,
                    axum::http::Method::OPTIONS,
                ]),
        );

    // Build router with auth service and JWT service as extensions
    Router::new()
        .route("/", get(graphql_playground))
        .route("/health", get(health_check))
        .route("/graphql", get(graphql_playground).post(graphql_handler))
        .layer(Extension(Arc::clone(&auth_service))) // Make Auth service available first
        .layer(Extension(Arc::clone(&jwt_service))) // Make JWT service available
        .layer(Extension(schema)) // Extension middleware for the schema
        .layer(middleware_stack) // Attach middleware stack to routes
        .layer(middleware::from_fn(error_handling_middleware)) // Apply custom error handling middleware
}
