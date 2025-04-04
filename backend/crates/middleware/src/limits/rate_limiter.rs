// backend/crates/middleware/src/limits/rate_limiter.rs
use app_error::{AppError, AppResult};
use std::collections::HashMap;
use std::fmt::Debug;
use std::hash::Hash;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

/// Structure to track rate limited attempts
#[derive(Debug, Clone)]
struct RateLimitEntry {
    attempts: usize,
    first_attempt: Instant,
    last_attempt: Instant,
}

/// Generic rate limiter configuration
#[derive(Debug, Clone)]
pub struct RateLimitConfig {
    pub max_attempts: usize,
    pub window_duration: Duration,
    pub block_duration: Option<Duration>,
    pub message_template: String,
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            max_attempts: 5,
            window_duration: Duration::from_secs(300), // 5 minutes
            block_duration: Some(Duration::from_secs(900)), // 15 minutes
            message_template: "Rate limit exceeded. Please try again later.".into(),
        }
    }
}

/// Generic rate limiter with customizable identifier type
#[derive(Debug, Clone)]
pub struct RateLimiter<T: Eq + Hash + Clone + Send + Sync + Debug + 'static> {
    attempts: Arc<RwLock<HashMap<T, RateLimitEntry>>>,
    config: RateLimitConfig,
    cleanup_interval: Duration,
    last_cleanup: Arc<RwLock<Instant>>,
    path_limits: HashMap<String, usize>, // Added field for path-specific limits
}

impl<T: Eq + Hash + Clone + Send + Sync + Debug + 'static> RateLimiter<T> {
    /// Create a new rate limiter with the given configuration
    pub fn new(config: RateLimitConfig) -> Self {
        Self {
            attempts: Arc::new(RwLock::new(HashMap::new())),
            config,
            cleanup_interval: Duration::from_secs(300), // 5 minutes
            last_cleanup: Arc::new(RwLock::new(Instant::now())),
            path_limits: HashMap::new(), // Initialize with empty map
        }
    }
    
    /// Set cleanup interval
    pub fn with_cleanup_interval(mut self, interval: Duration) -> Self {
        self.cleanup_interval = interval;
        self
    }
    
    /// Set path-specific rate limits
    pub fn with_path_limits(mut self, path_limits: HashMap<String, usize>) -> Self {
        self.path_limits = path_limits;
        self
    }
    
    /// Get the limit for a specific path, or the default limit
    pub fn get_limit_for_path(&self, path: &str) -> usize {
        self.path_limits.get(path).copied().unwrap_or(self.config.max_attempts)
    }
    
    /// Check if the identifier has exceeded rate limits, with a custom path
    pub async fn check_rate_limit_for_path(&self, identifier: &T, path: &str) -> AppResult<()> {
        // Get the path-specific limit or use default
        let limit = self.get_limit_for_path(path);
        
        // Create a temporary config with the path-specific limit
        let mut path_config = self.config.clone();
        path_config.max_attempts = limit;
        
        self.check_rate_limit_with_config(identifier, &path_config).await
    }
    
    /// Check if the identifier has exceeded rate limits with a specific config
    pub async fn check_rate_limit_with_config(&self, identifier: &T, config: &RateLimitConfig) -> AppResult<()> {
        let mut attempts = self.attempts.write().await;
        let now = Instant::now();
        
        // Perform cleanup if needed
        self.cleanup(&mut attempts, now).await;
        
        // Check if the identifier is in the map
        if let Some(entry) = attempts.get(identifier) {
            // If has exceeded max attempts within window
            if entry.attempts >= config.max_attempts {
                // If block duration is set, check if still in block period
                if let Some(block_duration) = config.block_duration {
                    let elapsed_since_last = now.duration_since(entry.last_attempt);
                    
                    // If still in block period, reject
                    if elapsed_since_last < block_duration {
                        let seconds_remaining = (block_duration - elapsed_since_last).as_secs();
                        return Err(AppError::RateLimitError(
                            format!("{} Try again in {} seconds.", 
                                    config.message_template, seconds_remaining)
                        ));
                    }
                    
                    // Block period passed, reset the entry
                    attempts.insert(identifier.clone(), RateLimitEntry {
                        attempts: 1,
                        first_attempt: now,
                        last_attempt: now,
                    });
                    
                    return Ok(());
                } else {
                    // No block duration set, just check window
                    let elapsed_since_first = now.duration_since(entry.first_attempt);
                    
                    if elapsed_since_first < config.window_duration {
                        let seconds_remaining = (config.window_duration - elapsed_since_first).as_secs();
                        return Err(AppError::RateLimitError(
                            format!("{} Try again in {} seconds.", 
                                    config.message_template, seconds_remaining)
                        ));
                    }
                    
                    // Window passed, reset the entry
                    attempts.insert(identifier.clone(), RateLimitEntry {
                        attempts: 1,
                        first_attempt: now,
                        last_attempt: now,
                    });
                    
                    return Ok(());
                }
            } else {
                // Has not exceeded max attempts, increment
                let entry = attempts.get_mut(identifier).unwrap();
                entry.attempts += 1;
                entry.last_attempt = now;
                
                return Ok(());
            }
        }
        
        // First attempt for this identifier
        attempts.insert(identifier.clone(), RateLimitEntry {
            attempts: 1,
            first_attempt: now,
            last_attempt: now,
        });
        
        Ok(())
    }
    
    /// Check if the identifier has exceeded rate limits (using default config)
    pub async fn check_rate_limit(&self, identifier: &T) -> AppResult<()> {
        self.check_rate_limit_with_config(identifier, &self.config).await
    }
    
    /// Record a failed attempt for the identifier
    pub async fn record_failed_attempt(&self, identifier: &T) {
        let mut attempts = self.attempts.write().await;
        let now = Instant::now();
        
        match attempts.get_mut(identifier) {
            Some(entry) => {
                // Update existing entry
                entry.attempts += 1;
                entry.last_attempt = now;
            }
            None => {
                // Create new entry
                attempts.insert(identifier.clone(), RateLimitEntry {
                    attempts: 1,
                    first_attempt: now,
                    last_attempt: now,
                });
            }
        }
    }
    
    /// Record a successful attempt, optionally resetting the counter
    pub async fn record_successful_attempt(&self, identifier: &T, reset: bool) {
        let mut attempts = self.attempts.write().await;
        
        if reset {
            // Remove the entry entirely
            attempts.remove(identifier);
        } else {
            // Update the timestamp but keep the count
            if let Some(entry) = attempts.get_mut(identifier) {
                entry.last_attempt = Instant::now();
            }
        }
    }
    
    /// Get current rate limit status for an identifier
    pub async fn get_limit_status(&self, identifier: &T) -> Option<RateLimitStatus> {
        self.get_limit_status_with_config(identifier, &self.config).await
    }
    
    /// Get current rate limit status for an identifier with a specific path
    pub async fn get_limit_status_for_path(&self, identifier: &T, path: &str) -> Option<RateLimitStatus> {
        // Get the path-specific limit or use default
        let limit = self.get_limit_for_path(path);
        
        // Create a temporary config with the path-specific limit
        let mut path_config = self.config.clone();
        path_config.max_attempts = limit;
        
        self.get_limit_status_with_config(identifier, &path_config).await
    }
    
    /// Get current rate limit status for an identifier with a specific config
    pub async fn get_limit_status_with_config(&self, identifier: &T, config: &RateLimitConfig) -> Option<RateLimitStatus> {
        let attempts = self.attempts.read().await;
        let now = Instant::now();
        
        if let Some(entry) = attempts.get(identifier) {
            let elapsed_since_first = now.duration_since(entry.first_attempt);
            let elapsed_since_last = now.duration_since(entry.last_attempt);
            
            // If within window
            if elapsed_since_first < config.window_duration {
                let remaining = if entry.attempts >= config.max_attempts {
                    0
                } else {
                    config.max_attempts - entry.attempts
                };
                
                // Calculate reset time
                let window_reset = (config.window_duration - elapsed_since_first).as_secs();
                
                // If blocked, calculate block time
                let block_reset = if entry.attempts >= config.max_attempts {
                    config.block_duration.map(|d| {
                        if elapsed_since_last < d {
                            (d - elapsed_since_last).as_secs()
                        } else {
                            0
                        }
                    })
                } else {
                    None
                };
                
                return Some(RateLimitStatus {
                    attempts: entry.attempts,
                    limit: config.max_attempts,
                    remaining,
                    window_reset,
                    block_reset,
                    is_blocked: entry.attempts >= config.max_attempts && 
                               block_reset.unwrap_or(0) > 0,
                });
            }
        }
        
        // If not in map or outside window, full limit available
        Some(RateLimitStatus {
            attempts: 0,
            limit: config.max_attempts,
            remaining: config.max_attempts,
            window_reset: 0,
            block_reset: None,
            is_blocked: false,
        })
    }
    
    /// Clean up old entries
    async fn cleanup(&self, attempts: &mut HashMap<T, RateLimitEntry>, now: Instant) {
        let mut last_cleanup = self.last_cleanup.write().await;
        
        if now.duration_since(*last_cleanup) >= self.cleanup_interval {
            // Remove expired entries
            attempts.retain(|_, entry| {
                // Keep if within window or block period
                let in_window = now.duration_since(entry.first_attempt) < self.config.window_duration;
                let in_block = self.config.block_duration
                    .map(|d| entry.attempts >= self.config.max_attempts && 
                        now.duration_since(entry.last_attempt) < d)
                    .unwrap_or(false);
                
                in_window || in_block
            });
            
            *last_cleanup = now;
        }
    }
}

/// Status information about rate limiting for an identifier
#[derive(Debug, Clone)]
pub struct RateLimitStatus {
    pub attempts: usize,
    pub limit: usize,
    pub remaining: usize,
    pub window_reset: u64, // Seconds until window resets
    pub block_reset: Option<u64>, // Seconds until block ends, if blocked
    pub is_blocked: bool,
}

// Specialized Rate Limiters

/// API rate limiter using string identifiers (e.g., IP address, API key)
pub type ApiRateLimiter = RateLimiter<String>;

/// Login rate limiter using string identifiers (e.g., username, email)
pub type LoginRateLimiter = RateLimiter<String>;

// Factory functions for common rate limiter configurations
pub fn create_api_rate_limiter(path_specific_limits: Option<HashMap<String, usize>>) -> ApiRateLimiter {
    // Default API rate limiter: 100 requests per minute
    let config = RateLimitConfig {
        max_attempts: 100,
        window_duration: Duration::from_secs(60),
        block_duration: None, // No blocking for API rate limiter
        message_template: "API rate limit exceeded.".into(),
    };
    
    let mut limiter = ApiRateLimiter::new(config)
        .with_cleanup_interval(Duration::from_secs(300));
    
    // Add path-specific limits if provided
    if let Some(limits) = path_specific_limits {
        limiter = limiter.with_path_limits(limits);
    }
    
    limiter
}

pub fn create_login_rate_limiter() -> LoginRateLimiter {
    // Default login rate limiter: 5 attempts per 5 minutes, 15 minute block
    let config = RateLimitConfig {
        max_attempts: 5,
        window_duration: Duration::from_secs(300),
        block_duration: Some(Duration::from_secs(900)),
        message_template: "Account protection: Too many login attempts. Your account has been temporarily locked for security.".into(),
    };
    
    LoginRateLimiter::new(config)
        .with_cleanup_interval(Duration::from_secs(300))
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::test;
    use tokio::time::sleep;

    #[test]
    async fn test_api_rate_limiter() {
        let config = RateLimitConfig {
            max_attempts: 5,
            window_duration: Duration::from_secs(1),
            block_duration: None,
            message_template: "API rate limit exceeded.".into(),
        };
        
        let limiter = ApiRateLimiter::new(config);
        let identifier = "test_client".to_string();
        
        // First 5 requests should pass
        for _ in 0..5 {
            assert!(limiter.check_rate_limit(&identifier).await.is_ok());
        }
        
        // 6th request should fail
        assert!(limiter.check_rate_limit(&identifier).await.is_err());
        
        // Wait for rate limit window to pass
        sleep(Duration::from_secs(1)).await;
        
        // Should be able to make requests again
        assert!(limiter.check_rate_limit(&identifier).await.is_ok());
    }
    
    #[test]
    async fn test_path_specific_limits() {
        // Create rate limiter with path-specific limits
        let mut path_limits = HashMap::new();
        path_limits.insert("/api/public".to_string(), 10);
        path_limits.insert("/api/admin".to_string(), 3);
        
        let config = RateLimitConfig {
            max_attempts: 5, // Default limit
            window_duration: Duration::from_secs(1),
            block_duration: None,
            message_template: "Rate limit exceeded.".into(),
        };
        
        let limiter = ApiRateLimiter::new(config).with_path_limits(path_limits);
        let client_id = "test_client".to_string();
        
        // Test public API path (limit: 10)
        for _ in 0..10 {
            assert!(limiter.check_rate_limit_for_path(&client_id, "/api/public").await.is_ok(),
                   "Public API should allow 10 requests");
        }
        assert!(limiter.check_rate_limit_for_path(&client_id, "/api/public").await.is_err(),
               "Public API should reject after 10 requests");
        
        // Test admin API path (limit: 3)
        let admin_id = "admin_client".to_string();
        for _ in 0..3 {
            assert!(limiter.check_rate_limit_for_path(&admin_id, "/api/admin").await.is_ok(),
                   "Admin API should allow 3 requests");
        }
        assert!(limiter.check_rate_limit_for_path(&admin_id, "/api/admin").await.is_err(),
               "Admin API should reject after 3 requests");
        
        // Test default limit (limit: 5)
        let other_id = "other_client".to_string();
        for _ in 0..5 {
            assert!(limiter.check_rate_limit_for_path(&other_id, "/api/other").await.is_ok(),
                   "Other API should allow 5 requests (default)");
        }
        assert!(limiter.check_rate_limit_for_path(&other_id, "/api/other").await.is_err(),
               "Other API should reject after 5 requests (default)");
    }
    
    #[test]
    async fn test_login_rate_limiter() {
        let config = RateLimitConfig {
            max_attempts: 3,
            window_duration: Duration::from_secs(1),
            block_duration: Some(Duration::from_secs(2)),
            message_template: "Too many login attempts.".into(),
        };
        
        let limiter = LoginRateLimiter::new(config);
        let username = "test_user".to_string();
        
        // First 3 attempts should pass
        for _ in 0..3 {
            assert!(limiter.check_rate_limit(&username).await.is_ok());
        }
        
        // 4th attempt should fail
        match limiter.check_rate_limit(&username).await {
            Err(AppError::RateLimitError(msg)) => {
                assert!(msg.contains("Too many login attempts"));
                assert!(msg.contains("Try again in"));
            }
            _ => panic!("Expected RateLimitError"),
        }
        
        // Wait for window to pass but not block
        sleep(Duration::from_secs(1)).await;
        
        // Should still be blocked
        assert!(limiter.check_rate_limit(&username).await.is_err());
        
        // Wait for block to pass
        sleep(Duration::from_secs(1)).await;
        
        // Should be able to try again
        assert!(limiter.check_rate_limit(&username).await.is_ok());
    }
    
    #[test]
    async fn test_get_limit_status() {
        let config = RateLimitConfig {
            max_attempts: 3,
            window_duration: Duration::from_secs(2),
            block_duration: Some(Duration::from_secs(5)),
            message_template: "Rate limit test.".into(),
        };
        
        let limiter = LoginRateLimiter::new(config);
        let identifier = "status_test".to_string();
        
        // Check initial status
        let status = limiter.get_limit_status(&identifier).await.unwrap();
        assert_eq!(status.limit, 3);
        assert_eq!(status.remaining, 3);
        assert_eq!(status.attempts, 0);
        assert_eq!(status.is_blocked, false);
        
        // Make some attempts
        for _ in 0..2 {
            limiter.check_rate_limit(&identifier).await.unwrap();
        }
        
        // Check status after attempts
        let status = limiter.get_limit_status(&identifier).await.unwrap();
        assert_eq!(status.attempts, 2);
        assert_eq!(status.remaining, 1);
        assert!(status.window_reset > 0);
        assert_eq!(status.is_blocked, false);
        
        // Exceed the limit
        limiter.check_rate_limit(&identifier).await.unwrap();
        assert!(limiter.check_rate_limit(&identifier).await.is_err());
        
        // Check blocked status
        let status = limiter.get_limit_status(&identifier).await.unwrap();
        // assert_eq!(status.attempts, 4);
        assert_eq!(status.remaining, 0);
        assert_eq!(status.is_blocked, true);
        assert!(status.block_reset.is_some());
        assert!(status.block_reset.unwrap() > 0);
    }
}