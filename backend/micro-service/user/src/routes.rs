// backend/micro-service/user/src/routes.rs
use crate::{
    handlers::graphql::{graphql_handler, graphql_playground, health_check},
    schema::ApiSchema, 
    service::{AuthService, AuthServiceTrait},
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
    routing::get,
};
use tower_http::limit::RequestBodyLimitLayer;

use app_middleware::{
    api_middleware::{
         api_rate_limit_middleware, jwt_auth_middleware, logging_middleware, security_headers_middleware
    },
    limits::rate_limiter::create_api_rate_limiter
};
use app_error::middleware_handling::error_handling_middleware;

pub fn create_routes(
    schema: ApiSchema, 
    auth_service: Arc<AuthService>
) -> Router {
    // Create JWT service
    let jwt_service = auth_service.get_jwt_service();

    // Create API rate limiter
    let api_rate_limiter = Arc::new(create_api_rate_limiter(None));

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

    // Build router with optimized middleware stack
    let app = Router::new()
        .route("/", get(graphql_playground))
        .route("/health", get(health_check))
        .route("/graphql", get(graphql_playground).post(graphql_handler));

    // Add Extensions
    let app = app
        .layer(Extension(schema))
        .layer(Extension(Arc::clone(&auth_service)))
        .layer(Extension(jwt_service.clone()))
        .layer(Extension(Arc::clone(&api_rate_limiter)));

    // Apply middleware in order
    let app = app
        .layer(axum::middleware::from_fn(error_handling_middleware))
        .layer(RequestBodyLimitLayer::new(1024 * 1024));

    // Apply custom middleware stacks
    let app = app
        .layer(axum::middleware::from_fn(logging_middleware))
        .layer(axum::middleware::from_fn(security_headers_middleware));

    // Use with_state method instead of direct middleware application
    let app = app
        .layer(axum::middleware::from_fn_with_state(api_rate_limiter, api_rate_limit_middleware))
        .layer(axum::middleware::from_fn_with_state(jwt_service, jwt_auth_middleware));

    // Apply global middleware stack
    app.layer(middleware_stack)
}
