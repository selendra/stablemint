use axum::{
    body::Body,
    http::{Request, Response, StatusCode},
    middleware::Next,
    response::IntoResponse,
};
use std::time::Instant;
use tracing::{error, info};

use crate::errors::{AppError, ErrorResponse};

// Middleware to handle errors and log requests
pub async fn error_handling_middleware(
    req: Request<Body>,
    next: Next,
) -> Result<impl IntoResponse, AppError> {
    let start = Instant::now();
    let path = req.uri().path().to_owned();
    let method = req.method().clone();

    info!("Request: {} {}", method, path);

    // Process the request
    let response = next.run(req).await;

    // Log request completion time
    let latency = start.elapsed();
    info!(
        "Request completed: {} {} - Status: {} - Time: {:?}",
        method,
        path,
        response.status(),
        latency
    );

    // Check if the response status indicates an error
    let status = response.status();
    if status.is_server_error() {
        error!("Server error occurred: {}", status);

        // For server errors, return a generic error response
        let error_response = ErrorResponse {
            status: status.to_string(),
            message: "An internal server error occurred".to_string(),
            details: None,
        };

        return Ok(Response::builder()
            .status(StatusCode::INTERNAL_SERVER_ERROR)
            .body(Body::from(serde_json::to_string(&error_response).unwrap()))
            .unwrap());
    }

    // If everything is fine, just return the original response
    Ok(response)
}
