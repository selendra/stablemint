use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use app_error::{AppError, AppResult};
use tokio::sync::RwLock;

/// Structure to track API request rates
#[derive(Debug)]
struct RequestTracker {
    count: usize,
    first_request: Instant,
    last_request: Instant,
}

#[derive(Debug)]
pub struct RateLimitInfo {
    pub limit: usize,
    pub remaining: usize,
    pub reset_time: u64, // Unix timestamp when the limit resets
}

/// Rate limiter for API requests
#[derive(Debug, Clone)]
pub struct ApiRateLimiter {
    requests: Arc<RwLock<HashMap<String, RequestTracker>>>,
    window_duration: Duration,
    max_requests: HashMap<String, usize>,  // Path-specific limits
    default_max_requests: usize,           // Default limit
    cleanup_interval: Duration,
    last_cleanup: Arc<RwLock<Instant>>,
}

impl ApiRateLimiter {
    /// Create a new API rate limiter
    pub fn new(
        window_duration: Duration,
        default_max_requests: usize,
        cleanup_interval: Duration,
    ) -> Self {
        Self {
            requests: Arc::new(RwLock::new(HashMap::new())),
            window_duration,
            max_requests: HashMap::new(),
            default_max_requests,
            cleanup_interval,
            last_cleanup: Arc::new(RwLock::new(Instant::now())),
        }
    }

    /// Create a default API rate limiter with sensible defaults
    pub fn default() -> Self {
        // Default: 100 requests per minute, cleanup every 5 minutes
        let mut limiter = Self::new(
            Duration::from_secs(60),
            100,
            Duration::from_secs(300),
        );
        
        // Add path-specific rate limits
        limiter.add_path_limit("/graphql", 60);  // 60 requests/minute for GraphQL
        limiter.add_path_limit("/login", 10);    // 10 login attempts/minute
        limiter.add_path_limit("/register", 5);  // 5 registrations/minute
        
        limiter
    }
    
    /// Add a rate limit for a specific path
    pub fn add_path_limit(&mut self, path: &str, max_requests: usize) -> &mut Self {
        self.max_requests.insert(path.to_string(), max_requests);
        self
    }

    /// Check if a client can make a request
    pub async fn check_rate_limit(&self, client_id: &str, path: &str) -> AppResult<()> {
        let mut requests = self.requests.write().await;
        let now = Instant::now();
        
        // Perform periodic cleanup
        self.maybe_cleanup(&mut requests, now).await;
        
        // Get path-specific rate limit or use default
        let limit = self.max_requests.get(path).unwrap_or(&self.default_max_requests);
        
        // Generate a key that combines client ID and path
        let key = format!("{}:{}", client_id, path);
        
        // Check if the client is in the map
        if let Some(tracker) = requests.get(&key) {
            // If within time window and over limit, reject
            if now.duration_since(tracker.first_request) <= self.window_duration 
                && tracker.count >= *limit {
                
                let reset_time = tracker.first_request + self.window_duration;
                let seconds_remaining = reset_time.duration_since(now).as_secs();
                
                return Err(AppError::RateLimitError(
                    format!("Rate limit exceeded. Please try again in {} seconds", seconds_remaining)
                ));
            }
            
            // If outside the window, reset
            if now.duration_since(tracker.first_request) > self.window_duration {
                requests.insert(key, RequestTracker {
                    count: 1,
                    first_request: now,
                    last_request: now,
                });
            } else {
                // Update existing record
                let tracker = requests.get_mut(&key).unwrap();
                tracker.count += 1;
                tracker.last_request = now;
            }
        } else {
            // Create new record
            requests.insert(key, RequestTracker {
                count: 1,
                first_request: now,
                last_request: now,
            });
        }
        
        Ok(())
    }
    
    /// Clean up old entries if needed
    async fn maybe_cleanup(&self, requests: &mut HashMap<String, RequestTracker>, now: Instant) {
        let mut last_cleanup = self.last_cleanup.write().await;
        
        if now.duration_since(*last_cleanup) >= self.cleanup_interval {
            // Remove expired entries
            requests.retain(|_, tracker| {
                now.duration_since(tracker.first_request) <= self.window_duration
            });
            
            *last_cleanup = now;
        }
    }

    pub async fn get_limit_info(&self, client_id: &str, path: &str) -> Option<RateLimitInfo> {
        let requests = self.requests.read().await;
        
        // Get path-specific rate limit or use default
        let limit = self.max_requests.get(path).unwrap_or(&self.default_max_requests);
        
        // Generate a key that combines client ID and path
        let key = format!("{}:{}", client_id, path);
        
        if let Some(tracker) = requests.get(&key) {
            // If within window, calculate remaining requests
            if Instant::now().duration_since(tracker.first_request) <= self.window_duration {
                let remaining = if tracker.count >= *limit {
                    0
                } else {
                    limit - tracker.count
                };
                
                // Calculate reset time
                let reset_time = (tracker.first_request + self.window_duration)
                    .duration_since(Instant::now())
                    .as_secs();
                
                return Some(RateLimitInfo {
                    limit: *limit,
                    remaining,
                    reset_time,
                });
            }
        }
        
        // If no record or outside window, full limit is available
        Some(RateLimitInfo {
            limit: *limit,
            remaining: *limit,
            reset_time: 0,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::test;
    use tokio::time::sleep;

    #[test]
    async fn test_rate_limiter_basic() {
        // Create a rate limiter with a short window (1 second) and 5 max requests
        let limiter = ApiRateLimiter::new(
            Duration::from_secs(1),
            5,
            Duration::from_secs(60),
        );

        let client_id = "test_client";
        let path = "/test_path";

        // First 5 requests should pass
        for _ in 0..5 {
            assert!(limiter.check_rate_limit(client_id, path).await.is_ok());
        }

        // 6th request should fail
        assert!(limiter.check_rate_limit(client_id, path).await.is_err());

        // Wait for rate limit window to pass
        sleep(Duration::from_secs(1)).await;

        // Should be able to make requests again
        assert!(limiter.check_rate_limit(client_id, path).await.is_ok());
    }

    #[test]
    async fn test_path_specific_limits() {
        // Create a rate limiter with different limits for different paths
        let mut limiter = ApiRateLimiter::new(
            Duration::from_secs(1),
            10, // Default limit
            Duration::from_secs(60),
        );
        
        limiter.add_path_limit("/restricted", 2);

        let client_id = "test_client";
        
        // For default path: first 10 requests should pass
        for _ in 0..10 {
            assert!(limiter.check_rate_limit(client_id, "/default").await.is_ok());
        }
        
        // 11th request should fail
        assert!(limiter.check_rate_limit(client_id, "/default").await.is_err());
        
        // For restricted path: first 2 requests should pass
        for _ in 0..2 {
            assert!(limiter.check_rate_limit(client_id, "/restricted").await.is_ok());
        }
        
        // 3rd request should fail
        assert!(limiter.check_rate_limit(client_id, "/restricted").await.is_err());
        
        // Different clients should have separate limits
        assert!(limiter.check_rate_limit("other_client", "/restricted").await.is_ok());
    }

    #[test]
    async fn test_cleanup() {
        // Create a rate limiter with short cleanup interval
        let limiter = ApiRateLimiter::new(
            Duration::from_millis(100), // Short window
            5,
            Duration::from_millis(200), // Short cleanup interval
        );

        let client_id = "test_client";
        let path = "/test_path";

        // Fill up the rate limit
        for _ in 0..5 {
            assert!(limiter.check_rate_limit(client_id, path).await.is_ok());
        }
        
        // Wait for window to expire
        sleep(Duration::from_millis(100)).await;
        
        // Make a request to trigger cleanup
        assert!(limiter.check_rate_limit("other_client", path).await.is_ok());
        
        // Wait for cleanup to happen
        sleep(Duration::from_millis(100)).await;
        
        // Check that limiter is cleaned up by checking if we can make more requests
        assert!(limiter.check_rate_limit(client_id, path).await.is_ok());
    }
}