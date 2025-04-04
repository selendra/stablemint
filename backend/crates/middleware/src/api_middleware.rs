use axum::{
    body::Body,
    extract::{Request, State},
    http::{HeaderMap, HeaderValue, header},
    middleware::Next,
    response::{IntoResponse, Response},
};
use std::{net::IpAddr, str::FromStr, sync::Arc};
use tracing::{warn, trace, info, error};

use crate::limits::rate_limiter::{ApiRateLimiter, RateLimitStatus, RateLimiter};
use crate::JwtService;

// Extract client identifier from request
pub fn extract_client_id(req: &Request<Body>) -> String {
    // First check for API key in header
    if let Some(api_key) = req.headers().get("X-API-Key") {
        if let Ok(key) = api_key.to_str() {
            return key.to_string();
        }
    }
    
    // If no API key, use IP address
    if let Some(ip) = get_client_ip(req) {
        return ip.to_string();
    }
    
    // Fallback to a default value
    "unknown".to_string()
}

// Get client IP from various headers or connection info
pub fn get_client_ip(req: &Request<Body>) -> Option<IpAddr> {
    // Try X-Forwarded-For header first (common for proxies)
    if let Some(forward) = req.headers().get("X-Forwarded-For") {
        if let Ok(forward_str) = forward.to_str() {
            if let Some(ip) = forward_str.split(',').next() {
                if let Ok(ip_addr) = IpAddr::from_str(ip.trim()) {
                    return Some(ip_addr);
                }
            }
        }
    }
    
    // Try X-Real-IP header (used by some proxies)
    if let Some(real_ip) = req.headers().get("X-Real-IP") {
        if let Ok(real_ip_str) = real_ip.to_str() {
            if let Ok(ip_addr) = IpAddr::from_str(real_ip_str.trim()) {
                return Some(ip_addr);
            }
        }
    }
    
    // Try to get the peer address from the connection
    req.extensions()
        .get::<axum::extract::connect_info::ConnectInfo<std::net::SocketAddr>>()
        .map(|connect_info| connect_info.ip())
}

// Add rate limit headers to response
pub fn add_rate_limit_headers(response: &mut Response, status: &RateLimitStatus) {
    let headers = response.headers_mut();
    
    headers.insert("X-RateLimit-Limit", 
        HeaderValue::from(status.limit));
    headers.insert("X-RateLimit-Remaining", 
        HeaderValue::from(status.remaining));
    headers.insert("X-RateLimit-Reset", 
        HeaderValue::from(status.window_reset));
    
    if let Some(block_reset) = status.block_reset {
        headers.insert("X-RateLimit-BlockReset", 
            HeaderValue::from(block_reset));
    }
}

// Unified API rate limiting middleware
pub async fn api_rate_limit_middleware(
    State(rate_limiter): State<Arc<ApiRateLimiter>>,
    req: Request<Body>,
    next: Next,
) -> Response {
    // Get client identifier and path
    let client_id = extract_client_id(&req);
    let path = req.uri().path().to_owned();
    
    // Create a combined identifier that includes the path
    let path_identifier = format!("{}:{}", client_id, path);
    
    // Get rate limit status
    let limit_status = rate_limiter.get_limit_status(&path_identifier).await;
    
    // Check rate limit
    match rate_limiter.check_rate_limit(&path_identifier).await {
        Ok(_) => {
            // Rate limit not exceeded, continue processing
            trace!("Rate limit check passed for client {} on path {}", client_id, path);
            let mut response = next.run(req).await;
            
            // Add rate limit headers to response if status available
            if let Some(status) = limit_status {
                add_rate_limit_headers(&mut response, &status);
            }
            
            response
        }
        Err(err) => {
            // Rate limit exceeded
            warn!("Rate limit exceeded for client {} on path {}", client_id, path);
            
            let mut response = err.into_response();
            
            // Add rate limit headers
            if let Some(status) = limit_status {
                add_rate_limit_headers(&mut response, &status);
            }
            
            response
        }
    }
}

// JWT authentication middleware
pub async fn jwt_auth_middleware(
    headers: HeaderMap,
    State(jwt_service): State<Arc<JwtService>>,
    mut req: Request<Body>,
    next: Next,
) -> Response {
    if let Some(auth_header) = headers.get(header::AUTHORIZATION) {
        if let Ok(auth_str) = auth_header.to_str() {
            if auth_str.starts_with("Bearer ") {
                let token = &auth_str["Bearer ".len()..];

                match jwt_service.validate_token(token) {
                    Ok(claims) => {
                        info!("JWT validated for user {}", claims.username);
                        // Insert the claims into request extensions so handlers can access it
                        req.extensions_mut().insert(claims);
                    }
                    Err(e) => {
                        warn!("JWT validation failed: {}", e);
                        // Continue without authenticated user
                    }
                }
            }
        }
    }

    next.run(req).await
}

// Security headers middleware
pub async fn security_headers_middleware(
    req: Request<Body>,
    next: Next,
) -> Response {
    let mut response = next.run(req).await;
    
    // Add security headers
    let headers = response.headers_mut();
    
    // Basic security headers
    headers.insert(
        "X-Content-Type-Options", 
        HeaderValue::from_static("nosniff")
    );
    
    headers.insert(
        "X-Frame-Options", 
        HeaderValue::from_static("DENY")
    );
    
    headers.insert(
        "X-XSS-Protection", 
        HeaderValue::from_static("1; mode=block")
    );
    
    headers.insert(
        "Referrer-Policy", 
        HeaderValue::from_static("strict-origin-when-cross-origin")
    );
    
    // Content Security Policy - adjust as needed
    headers.insert(
        "Content-Security-Policy", 
        HeaderValue::from_static("default-src 'self'; script-src 'self'; connect-src 'self';")
    );
    
    response
}

// Logging middleware with performance tracking
pub async fn logging_middleware(
    req: Request<Body>,
    next: Next,
) -> Response {
    use std::time::Instant;
    
    let start = Instant::now();
    let method = req.method().clone();
    let path = req.uri().path().to_owned();
    let client_id = extract_client_id(&req);
    
    // Log request start
    info!(
        method = %method,
        path = %path,
        client = %client_id,
        "Request started"
    );
    
    // Process the request
    let response = next.run(req).await;
    
    // Calculate request duration
    let duration = start.elapsed();
    let status = response.status().as_u16();
    
    // Log request completion with appropriate level based on status
    if status < 400 {
        info!(
            method = %method,
            path = %path,
            client = %client_id,
            status = %status,
            duration_ms = %duration.as_millis(),
            "Request completed"
        );
    } else if status < 500 {
        warn!(
            method = %method,
            path = %path,
            client = %client_id,
            status = %status,
            duration_ms = %duration.as_millis(),
            "Request completed with client error"
        );
    } else {
        error!(
            method = %method,
            path = %path,
            client = %client_id,
            status = %status,
            duration_ms = %duration.as_millis(),
            "Request completed with server error"
        );
    }
    
    response
}

// Combined API middleware stack - for convenience
pub fn api_middleware_stack(rate_limiter: Arc<ApiRateLimiter>) -> impl tower::Layer<axum::extract::Request<Body>> + Clone {
    axum::middleware::from_fn_with_state::<_, Arc<RateLimiter<String>>, Body>(rate_limiter, api_rate_limit_middleware)
}

// Combined JWT middleware stack - for convenience
pub fn jwt_middleware_stack(jwt_service: Arc<JwtService>) -> impl tower::Layer<axum::extract::Request<Body>> + Clone {
    axum::middleware::from_fn_with_state::<_, Arc<JwtService>, Body>(jwt_service, jwt_auth_middleware)
}

// Combined security headers middleware
pub fn security_middleware_stack() -> impl tower::Layer<axum::extract::Request<Body>> + Clone {
    axum::middleware::from_fn::<_, Body>(security_headers_middleware)
}

// Combined logging middleware
pub fn logging_middleware_stack() -> impl tower::Layer<axum::extract::Request<Body>> + Clone {
    axum::middleware::from_fn::<_, Body>(logging_middleware)
}
