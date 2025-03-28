use crate::handlers::graphql::{graphql_handler, graphql_playground, health_check};
use crate::middleware::error::error_handling_middleware;
use crate::schema::ApiSchema;
use axum::{Router, extract::Extension, middleware, routing::get};
use std::time::Duration;
use tower::ServiceBuilder;
use tower_http::{
    compression::CompressionLayer,
    cors::{Any, CorsLayer}, // CORS middleware
    timeout::TimeoutLayer,
    trace::TraceLayer,
};

// Create application routes with middleware
pub fn create_routes(schema: ApiSchema) -> Router {
    // Define global middleware stack
    let middleware_stack = ServiceBuilder::new()
        .layer(TraceLayer::new_for_http())
        .layer(CompressionLayer::new())
        .layer(TimeoutLayer::new(Duration::from_secs(30)))
        .layer(CorsLayer::new().allow_origin(Any));

    Router::new()
        .route("/", get(graphql_playground))
        .route("/health", get(health_check))
        .route("/graphql", get(graphql_playground).post(graphql_handler))
        .layer(middleware_stack) // Attach middleware stack to routes
        .layer(middleware::from_fn(error_handling_middleware)) // Apply custom error handling middleware
        .layer(Extension(schema)) // Extension middleware for the schema
}
