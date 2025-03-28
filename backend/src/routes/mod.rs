use crate::{
    handlers::{
        auth::AuthService,
        graphql::{graphql_handler, graphql_playground, health_check},
    },
    middleware::{debug::debug_extensions, error::error_handling_middleware},
    schema::ApiSchema,
};
use axum::{
    Router,
    extract::Extension,
    middleware::{self},
    routing::get,
};
use std::{sync::Arc, time::Duration};
use tower::ServiceBuilder;
use tower_http::{
    compression::CompressionLayer,
    cors::{Any, CorsLayer},
    timeout::TimeoutLayer,
    trace::TraceLayer,
};

// Create application routes with middleware
pub fn create_routes(schema: ApiSchema, auth_service: Arc<AuthService>) -> Router {
    // Create JWT service
    let jwt_service = auth_service.get_jwt_service();

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
        .layer(middleware::from_fn(debug_extensions)) // Add debug middleware first
        .layer(Extension(Arc::clone(&auth_service))) // Make Auth service available
        .layer(Extension(jwt_service)) // Make JWT service available
        .layer(Extension(schema)) // Extension middleware for the schema
        .layer(middleware_stack) // Attach middleware stack to routes
        .layer(middleware::from_fn(error_handling_middleware)) // Apply custom error handling middleware
}
