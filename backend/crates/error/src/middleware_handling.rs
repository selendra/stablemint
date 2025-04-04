use axum::{
    body::Body,
    http::{header, Request, Response, StatusCode},
    middleware::Next,
    response::IntoResponse,
};
use std::time::Instant;
use tracing::{error, info};

use crate::{AppError, ErrorResponse};

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

    // Handle specific error conditions
    let status = response.status();
    
    if status == StatusCode::PAYLOAD_TOO_LARGE {
        error!("Request body too large: {}", status);
        
        let error_response = ErrorResponse {
            status: status.to_string(),
            message: "The request body exceeds the maximum allowed size".to_string(),
            code: "PAYLOAD_TOO_LARGE".to_string(),
            details: Some("Please reduce the size of your request and try again".to_string()),
            help: Some("The maximum allowed request size is 5MB".to_string()),
        };

        return Ok(Response::builder()
            .status(StatusCode::PAYLOAD_TOO_LARGE)
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from(serde_json::to_string(&error_response).unwrap()))
            .unwrap());
    }
    
    if status.is_server_error() {
        error!("Server error occurred: {}", status);

        let error_response = ErrorResponse {
            status: status.to_string(),
            message: "An internal server error occurred".to_string(),
            code: "SERVER_ERROR".to_string(),
            details: None,
            help: Some("Please try again later or contact support if the issue persists".to_string()),
        };

        return Ok(Response::builder()
            .status(StatusCode::INTERNAL_SERVER_ERROR)
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from(serde_json::to_string(&error_response).unwrap()))
            .unwrap());
    }

    // If everything is fine, just return the original response
    Ok(response)
}
