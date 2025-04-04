use crate::{
    debug::debug_extensions,
    handlers::graphql::{graphql_handler, graphql_playground, health_check},
    schema::ApiSchema, service::{AuthService, AuthServiceTrait},
};
use std::{sync::Arc, time::Duration};
use tower::ServiceBuilder;
use tower_http::{
    compression::CompressionLayer,
    cors::{Any, CorsLayer},
    timeout::TimeoutLayer,
    trace::TraceLayer,
};

use axum::{
    Router,
    extract::Extension,
    middleware::{self},
    routing::get,
};
use tower_http::limit::RequestBodyLimitLayer;

use app_middleware::{api_rate_limiter::ApiRateLimiter, rate_limit::api_rate_limit_middleware};
use app_error::middleware_handling::error_handling_middleware;

pub fn create_routes(
    schema: ApiSchema, 
    auth_service: Arc<AuthService>
) -> Router {
    // Create JWT service
    let jwt_service = auth_service.get_jwt_service();

     // Create API rate limiter
     let api_rate_limiter = Arc::new(ApiRateLimiter::default());

    // Define global middleware stack WITHOUT the body limit
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

    // Build router with all middleware
    Router::new()
        .route("/", get(graphql_playground))
        .route("/health", get(health_check))
        .route("/graphql", get(graphql_playground).post(graphql_handler))
        // Apply body limit directly to the router
        .layer(RequestBodyLimitLayer::new(1024 * 1024)) // 1 MB limit
        .layer(Extension(Arc::clone(&api_rate_limiter)))
        .layer(middleware::from_fn_with_state(
            Arc::clone(&api_rate_limiter),
            api_rate_limit_middleware
        ))
        .layer(middleware::from_fn(debug_extensions)) // Add debug middleware
        .layer(Extension(Arc::clone(&auth_service))) // Make Auth service available
        .layer(Extension(jwt_service)) // Make JWT service available
        .layer(Extension(schema)) // Extension middleware for the schema
        .layer(middleware_stack) // Attach middleware stack to routes
        .layer(middleware::from_fn(error_handling_middleware)) // Apply custom error handling middleware
}