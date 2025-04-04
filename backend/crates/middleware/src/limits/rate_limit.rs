use std::net::IpAddr;
use std::str::FromStr;
use std::sync::Arc;
use axum::http::header;
use axum::response::IntoResponse;
use axum::{
    body::Body,
    extract::Request,
    middleware::Next,
    response::Response,
};
use tracing::{warn, trace};

use crate::api_rate_limiter::ApiRateLimiter;

use super::api_rate_limiter::RateLimitInfo;

pub async fn api_rate_limit_middleware(
    req: Request<Body>,
    rate_limiter: Arc<ApiRateLimiter>,
    next: Next,
) -> Response {
    // Get client identifier and path
    let client_id = extract_client_id(&req);
    let path = req.uri().path().to_owned();
    
    // Get limit info from rate limiter
    let limit_info = rate_limiter.get_limit_info(&client_id, &path).await;
    
    // Check rate limit
    match rate_limiter.check_rate_limit(&client_id, &path).await {
        Ok(_) => {
            // Rate limit not exceeded, continue processing
            trace!("Rate limit check passed for client {} on path {}", client_id, path);
            let mut response = next.run(req).await;
            
            // Add rate limit headers to response
            if let Some(info) = limit_info {
                add_rate_limit_headers(&mut response, &info);
            }
            
            response
        }
        Err(err) => {
            // Rate limit exceeded, convert to response with appropriate headers
            warn!("Rate limit exceeded for client {} on path {}", client_id, path);
            
            let mut response = err.into_response();
            
            // Add rate limit headers
            if let Some(info) = limit_info {
                add_rate_limit_headers(&mut response, &info);
            }
            
            response
        }
    }
}

// Add rate limit headers to response
fn add_rate_limit_headers(response: &mut Response, info: &RateLimitInfo) {
    let headers = response.headers_mut();
    
    headers.insert("X-RateLimit-Limit", 
        header::HeaderValue::from(info.limit));
    headers.insert("X-RateLimit-Remaining", 
        header::HeaderValue::from(info.remaining));
    headers.insert("X-RateLimit-Reset", 
        header::HeaderValue::from(info.reset_time));
}

// Extract client identifier from request
fn extract_client_id(req: &Request<Body>) -> String {
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
fn get_client_ip(req: &Request<Body>) -> Option<IpAddr> {
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
    
    // Try to get the peer address from the connection (may not be available in all setups)
    req.extensions()
        .get::<axum::extract::connect_info::ConnectInfo<std::net::SocketAddr>>()
        .map(|connect_info| connect_info.ip())
    
    // Note: In production with multiple layers of proxies, you might need
    // additional logic to extract the correct client IP
}