use app_error::{AppError, AppResult};
use chrono::Utc;
use redis::{aio::ConnectionManager, AsyncCommands, Client, Pipeline};
use std::{
    collections::HashMap,
    fmt::Debug,
    hash::Hash,
    sync::Arc,
    time::{Duration, Instant},
};
use tokio::sync::RwLock;
use tracing::{debug, error, info};

/// Structure to track rate limited attempts status for UI/API responses
#[derive(Debug, Clone)]
pub struct RateLimitStatus {
    pub attempts: usize,
    pub limit: usize,
    pub remaining: usize,
    pub window_reset: i64, // Seconds until window resets
    pub block_reset: Option<i64>, // Seconds until block ends, if blocked
    pub is_blocked: bool,
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

/// Key prefix for rate limiting in Redis
const RATE_LIMIT_PREFIX: &str = "rate_limit";
const RATE_COUNT_SUFFIX: &str = "count";
const RATE_FIRST_SUFFIX: &str = "first";
const RATE_LAST_SUFFIX: &str = "last";
const RATE_BLOCK_SUFFIX: &str = "blocked_until";

/// Distributed rate limiter using Redis for shared state
#[derive(Clone)]
pub struct RedisRateLimiter<T: Eq + Hash + Clone + Send + Sync + Debug + 'static> {
    redis_manager: ConnectionManager,
    config: RateLimitConfig,
    last_cleanup: Arc<RwLock<Instant>>,
    cleanup_interval: Duration,
    path_limits: HashMap<String, usize>,
    _marker: std::marker::PhantomData<T>,
}

impl<T: Eq + Hash + Clone + Send + Sync + Debug + 'static> RedisRateLimiter<T> {
    /// Create a new rate limiter with Redis backend
    pub async fn new(redis_url: &str, config: RateLimitConfig) -> AppResult<Self> {
        let client = Client::open(redis_url).map_err(|e| {
            error!("Failed to connect to Redis: {}", e);
            AppError::ConfigError(anyhow::anyhow!("Redis connection failed: {}", e))
        })?;

        let manager = ConnectionManager::new(client).await.map_err(|e| {
            error!("Failed to create Redis connection manager: {}", e);
            AppError::ConfigError(anyhow::anyhow!("Redis connection manager failed: {}", e))
        })?;

        info!("Successfully connected to Redis for distributed rate limiting");

        Ok(Self {
            redis_manager: manager,
            config,
            last_cleanup: Arc::new(RwLock::new(Instant::now())),
            cleanup_interval: Duration::from_secs(300), // 5 minutes default cleanup
            path_limits: HashMap::new(),
            _marker: std::marker::PhantomData,
        })
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
        self.path_limits
            .get(path)
            .copied()
            .unwrap_or(self.config.max_attempts)
    }

    /// Convert identifier to Redis key
    fn get_rate_limit_key(&self, identifier: &T) -> String {
        format!("{}:{:?}", RATE_LIMIT_PREFIX, identifier)
    }

    /// Check if the identifier has exceeded rate limits, with a custom path
    pub async fn check_rate_limit_for_path(
        &self,
        identifier: &T,
        path: &str,
    ) -> AppResult<()> {
        // Get the path-specific limit or use default
        let limit = self.get_limit_for_path(path);

        // Create a temporary config with the path-specific limit
        let mut path_config = self.config.clone();
        path_config.max_attempts = limit;

        self.check_rate_limit_with_config(identifier, &path_config).await
    }

    /// Check if the identifier has exceeded rate limits with a specific config
    pub async fn check_rate_limit_with_config(
        &self,
        identifier: &T,
        config: &RateLimitConfig,
    ) -> AppResult<()> {
        let now = Utc::now().timestamp(); // i64
        let key_base = self.get_rate_limit_key(identifier);
        let count_key = format!("{}:{}", key_base, RATE_COUNT_SUFFIX);
        let first_key = format!("{}:{}", key_base, RATE_FIRST_SUFFIX);
        let last_key = format!("{}:{}", key_base, RATE_LAST_SUFFIX);
        let block_key = format!("{}:{}", key_base, RATE_BLOCK_SUFFIX);

        // Perform cleanup if needed
        self.cleanup_if_needed().await;

        // Get a Redis connection
        let mut conn = self.redis_manager.clone();

        // First check if the identifier is blocked
        let blocked_until: Option<i64> = conn.get(&block_key).await.unwrap_or(None);

        if let Some(blocked_until) = blocked_until {
            if now < blocked_until {
                let seconds_remaining = blocked_until - now;
                return Err(AppError::RateLimitError(format!(
                    "{} Try again in {} seconds.",
                    config.message_template, seconds_remaining
                )));
            }
            // Block expired, remove it
            let _: () = conn.del(&block_key).await.unwrap_or(());
        }

        // Get current count and timestamps using pipeline for efficiency
        let pipeline_result: Vec<Option<String>> = redis::pipe()
            .get(&count_key)
            .get(&first_key)
            .query_async(&mut conn)
            .await
            .map_err(|e| {
                error!("Redis pipeline error when getting count and timestamp: {}", e);
                AppError::ServerError(anyhow::anyhow!("Rate limit tracking error"))
            })?;

        let count: Option<usize> = pipeline_result[0].as_ref().and_then(|v| v.parse().ok());
        let first_attempt: Option<i64> = pipeline_result[1].as_ref().and_then(|v| v.parse().ok());

        // If first attempt exists, check if it's within the window
        if let (Some(count), Some(first)) = (count, first_attempt) {
            let window_secs = config.window_duration.as_secs() as i64; // Convert u64 to i64 only once
            let elapsed = now - first;

            // If window expired, reset counters
            if elapsed >= window_secs {
                // Window passed, reset counts
                let mut pipe = Pipeline::new();
                pipe.set(&count_key, 1)
                    .set(&first_key, now)
                    .set(&last_key, now)
                    .expire(&count_key, window_secs as i64) // Convert back to u64 for Redis
                    .expire(&first_key, window_secs as i64)
                    .expire(&last_key, window_secs as i64);

                let _: () = pipe.query_async(&mut conn).await.map_err(|e| {
                    error!("Redis pipeline error when resetting counters: {}", e);
                    AppError::ServerError(anyhow::anyhow!("Rate limit tracking error"))
                })?;

                return Ok(());
            }

            // If within window and exceeded attempts
            if count >= config.max_attempts {
                // If block duration is set, apply it
                if let Some(block_duration) = config.block_duration {
                    let block_secs = block_duration.as_secs() as i64; // Convert to i64 for timestamp math
                    let block_until = now + block_secs;

                    // Set blocked status
                    let _: () = conn
                        .set_ex(&block_key, block_until, block_secs as u64) // Convert back to u64 for Redis
                        .await
                        .map_err(|e| {
                            error!("Redis error when setting block: {}", e);
                            AppError::ServerError(anyhow::anyhow!("Rate limit tracking error"))
                        })?;

                    return Err(AppError::RateLimitError(format!(
                        "{} Try again in {} seconds.",
                        config.message_template, block_secs
                    )));
                }

                // No block duration, just reject until window expires
                let remaining_secs = window_secs - elapsed;
                return Err(AppError::RateLimitError(format!(
                    "{} Try again in {} seconds.",
                    config.message_template, remaining_secs
                )));
            }

            // Increment the counter
            let new_count: usize = conn.incr(&count_key, 1).await.map_err(|e| {
                error!("Redis error when incrementing counter: {}", e);
                AppError::ServerError(anyhow::anyhow!("Rate limit tracking error"))
            })?;

            // Update last attempt timestamp
            let _: () = conn
                .set_ex(&last_key, now, window_secs as u64) // Convert to u64 for Redis
                .await
                .map_err(|e| {
                    error!("Redis error when updating last attempt: {}", e);
                    AppError::ServerError(anyhow::anyhow!("Rate limit tracking error"))
                })?;

            debug!(
                "Rate limit increment for {:?}: {}/{}",
                identifier, new_count, config.max_attempts
            );

            return Ok(());
        }

        // First attempt for this identifier
        let mut pipe = Pipeline::new();
        let window_secs = config.window_duration.as_secs() as i64; // Convert only once
        pipe.set(&count_key, 1)
            .set(&first_key, now)
            .set(&last_key, now)
            .expire(&count_key, window_secs as i64) // Convert to u64 for Redis
            .expire(&first_key, window_secs as i64)
            .expire(&last_key, window_secs as i64);

        let _: () = pipe.query_async(&mut conn).await.map_err(|e| {
            error!("Redis pipeline error when setting initial counters: {}", e);
            AppError::ServerError(anyhow::anyhow!("Rate limit tracking error"))
        })?;

        debug!("Created new rate limit entry for {:?}", identifier);
        Ok(())
    }

    /// Check if the identifier has exceeded rate limits (using default config)
    pub async fn check_rate_limit(&self, identifier: &T) -> AppResult<()> {
        self.check_rate_limit_with_config(identifier, &self.config).await
    }

    /// Record a failed attempt for the identifier
    pub async fn record_failed_attempt(&self, identifier: &T) -> AppResult<()> {
        let now = Utc::now().timestamp(); // i64
        let key_base = self.get_rate_limit_key(identifier);
        let count_key = format!("{}:{}", key_base, RATE_COUNT_SUFFIX);
        let first_key = format!("{}:{}", key_base, RATE_FIRST_SUFFIX);
        let last_key = format!("{}:{}", key_base, RATE_LAST_SUFFIX);
        let window_secs = self.config.window_duration.as_secs() as i64; // Convert once

        // Get a Redis connection
        let mut conn = self.redis_manager.clone();

        // Use pipeline to check if keys exist and get values
        let results: Vec<Option<String>> = redis::pipe()
            .get(&count_key)
            .get(&first_key)
            .query_async(&mut conn)
            .await
            .map_err(|e| {
                error!("Redis pipeline error when getting values: {}", e);
                AppError::ServerError(anyhow::anyhow!("Rate limit tracking error"))
            })?;

        let count: Option<usize> = results[0].as_ref().and_then(|v| v.parse().ok());
        let first_attempt: Option<i64> = results[1].as_ref().and_then(|v| v.parse().ok());

        let mut pipe = Pipeline::new();

        if count.is_none() || first_attempt.is_none() {
            // Create new entry
            pipe.set(&count_key, 1)
                .set(&first_key, now)
                .set(&last_key, now)
                .expire(&count_key, window_secs as i64) // Convert to u64 for Redis
                .expire(&first_key, window_secs as i64)
                .expire(&last_key, window_secs as i64);
        } else {
            // Increment existing
            pipe.incr(&count_key, 1)
                .set_ex(&last_key, now, window_secs as u64); // Convert to u64 for Redis
        }

        let _: () = pipe.query_async(&mut conn).await.map_err(|e| {
            error!("Redis pipeline error when recording failed attempt: {}", e);
            AppError::ServerError(anyhow::anyhow!("Rate limit tracking error"))
        })?;

        Ok(())
    }

    /// Record a successful attempt, optionally resetting the counter
    pub async fn record_successful_attempt(&self, identifier: &T, reset: bool) -> AppResult<()> {
        if !reset {
            // If not resetting, we just update the last timestamp
            let now = Utc::now().timestamp(); // i64
            let key_base = self.get_rate_limit_key(identifier);
            let last_key = format!("{}:{}", key_base, RATE_LAST_SUFFIX);
            let window_secs = self.config.window_duration.as_secs() as i64; // Convert once

            // Get a Redis connection
            let mut conn = self.redis_manager.clone();

            // Update last attempt timestamp
            let _: () = conn
                .set_ex(&last_key, now, window_secs as u64) // Convert to u64 for Redis
                .await
                .map_err(|e| {
                    error!("Redis error when updating last attempt: {}", e);
                    AppError::ServerError(anyhow::anyhow!("Rate limit tracking error"))
                })?;

            return Ok(());
        }

        // If resetting, delete all related keys
        let key_base = self.get_rate_limit_key(identifier);
        let count_key = format!("{}:{}", key_base, RATE_COUNT_SUFFIX);
        let first_key = format!("{}:{}", key_base, RATE_FIRST_SUFFIX);
        let last_key = format!("{}:{}", key_base, RATE_LAST_SUFFIX);
        let block_key = format!("{}:{}", key_base, RATE_BLOCK_SUFFIX);

        // Get a Redis connection
        let mut conn = self.redis_manager.clone();

        // Delete all related keys
        let mut pipe = Pipeline::new();
        pipe.del(&count_key)
            .del(&first_key)
            .del(&last_key)
            .del(&block_key);

        let _: () = pipe.query_async(&mut conn).await.map_err(|e| {
            error!("Redis pipeline error when resetting rate limit: {}", e);
            AppError::ServerError(anyhow::anyhow!("Rate limit tracking error"))
        })?;

        Ok(())
    }

    /// Get current rate limit status for an identifier
    pub async fn get_limit_status(&self, identifier: &T) -> Option<RateLimitStatus> {
        self.get_limit_status_with_config(identifier, &self.config).await
    }

    /// Get current rate limit status for an identifier with a specific path
    pub async fn get_limit_status_for_path(
        &self,
        identifier: &T,
        path: &str,
    ) -> Option<RateLimitStatus> {
        // Get the path-specific limit or use default
        let limit = self.get_limit_for_path(path);

        // Create a temporary config with the path-specific limit
        let mut path_config = self.config.clone();
        path_config.max_attempts = limit;

        self.get_limit_status_with_config(identifier, &path_config).await
    }

    /// Get current rate limit status for an identifier with a specific config
    pub async fn get_limit_status_with_config(
        &self,
        identifier: &T,
        config: &RateLimitConfig,
    ) -> Option<RateLimitStatus> {
        let now = Utc::now().timestamp(); // i64
        let key_base = self.get_rate_limit_key(identifier);
        let count_key = format!("{}:{}", key_base, RATE_COUNT_SUFFIX);
        let first_key = format!("{}:{}", key_base, RATE_FIRST_SUFFIX);
        let block_key = format!("{}:{}", key_base, RATE_BLOCK_SUFFIX);

        // Get a Redis connection
        let mut conn = self.redis_manager.clone();

        // Get values with pipelining
        let results: Vec<Option<String>> = match redis::pipe()
            .get(&count_key)
            .get(&first_key)
            .get(&block_key)
            .query_async(&mut conn)
            .await
        {
            Ok(results) => results,
            Err(e) => {
                error!("Redis pipeline error when getting limit status: {}", e);
                return None;
            }
        };

        let count: Option<usize> = results[0].as_ref().and_then(|v| v.parse().ok());
        let first_attempt: Option<i64> = results[1].as_ref().and_then(|v| v.parse().ok());
        let blocked_until: Option<i64> = results[2].as_ref().and_then(|v| v.parse().ok());

        if let Some(count) = count {
            if let Some(first) = first_attempt {
                let window_secs = config.window_duration.as_secs() as i64; // Convert once
                let elapsed = now - first;

                // If still within window
                if elapsed < window_secs {
                    let remaining = if count >= config.max_attempts {
                        0
                    } else {
                        config.max_attempts - count
                    };

                    // Calculate reset time
                    let window_reset = window_secs - elapsed;

                    // Check if blocked
                    let (block_reset, is_blocked) = if let Some(blocked_until) = blocked_until {
                        if now < blocked_until {
                            (Some(blocked_until - now), true)
                        } else {
                            (None, false)
                        }
                    } else {
                        (None, false)
                    };

                    return Some(RateLimitStatus {
                        attempts: count,
                        limit: config.max_attempts,
                        remaining,
                        window_reset,
                        block_reset,
                        is_blocked,
                    });
                }
            }
        }

        // If not in database or outside window, full limit available
        Some(RateLimitStatus {
            attempts: 0,
            limit: config.max_attempts,
            remaining: config.max_attempts,
            window_reset: 0,
            block_reset: None,
            is_blocked: false,
        })
    }

    /// Clean up old entries if needed
    async fn cleanup_if_needed(&self) {
        let now = Instant::now();
        
        // Use try_write to avoid blocking if another task is doing cleanup
        if let Ok(mut last_cleanup) = self.last_cleanup.try_write() {
            if now.duration_since(*last_cleanup) >= self.cleanup_interval {
                debug!("Starting Redis rate limiter periodic cleanup");
                *last_cleanup = now;
                // Redis auto-expires keys with TTL, so no manual cleanup needed
            }
        }
    }
}

/// API rate limiter using string identifiers (e.g., IP address, API key)
pub type RedisApiRateLimiter = RedisRateLimiter<String>;

/// Login rate limiter using string identifiers (e.g., username, email)
pub type RedisLoginRateLimiter = RedisRateLimiter<String>;

/// Factory function for API rate limiter
pub async fn create_redis_api_rate_limiter(
    redis_url: &str,
    path_specific_limits: Option<HashMap<String, usize>>,
) -> AppResult<RedisApiRateLimiter> {
    // Default API rate limiter: 100 requests per minute
    let config = RateLimitConfig {
        max_attempts: 100,
        window_duration: Duration::from_secs(60),
        block_duration: None, // No blocking for API rate limiter
        message_template: "API rate limit exceeded.".into(),
    };

    let mut limiter = RedisApiRateLimiter::new(redis_url, config)
        .await?
        .with_cleanup_interval(Duration::from_secs(300));

    // Add path-specific limits if provided
    if let Some(limits) = path_specific_limits {
        limiter = limiter.with_path_limits(limits);
    }

    Ok(limiter)
}

/// Factory function for login rate limiter
pub async fn create_redis_login_rate_limiter(redis_url: &str) -> AppResult<RedisLoginRateLimiter> {
    // Default login rate limiter: 5 attempts per 5 minutes, 15 minute block
    let config = RateLimitConfig {
        max_attempts: 5,
        window_duration: Duration::from_secs(300),
        block_duration: Some(Duration::from_secs(900)),
        message_template: "Account protection: Too many login attempts. Your account has been temporarily locked for security.".into(),
    };

    Ok(RedisLoginRateLimiter::new(redis_url, config)
        .await?
        .with_cleanup_interval(Duration::from_secs(300)))
}


#[cfg(test)]
mod integration_tests {
    use super::*;
    use tokio::time::{sleep, Duration as TokioDuration};
    use std::env;
    use uuid::Uuid;

    // This test requires a running Redis server
    // It will be skipped if REDIS_URL environment variable is not set
    #[tokio::test]
    async fn test_rate_limiter_integration() {
        let redis_url =  "http://localhost:6379";

        // Create a unique test identifier to avoid collision with other tests
        let test_id = format!("test-{}", Uuid::new_v4());
        
        // Create a configuration with small durations for testing
        let config = RateLimitConfig {
            max_attempts: 3,
            window_duration: Duration::from_secs(3),
            block_duration: Some(Duration::from_secs(5)),
            message_template: "Test rate limit exceeded".into(),
        };

        // Create a rate limiter
        let rate_limiter = match RedisRateLimiter::<String>::new(&redis_url, config).await {
            Ok(rl) => rl,
            Err(e) => {
                println!("Failed to create rate limiter: {:?}", e);
                return;
            }
        };

        // Test basic rate limiting
        // First attempt should succeed
        let result = rate_limiter.check_rate_limit(&test_id).await;
        assert!(result.is_ok(), "First attempt should succeed");

        // Get status after first attempt
        if let Some(status) = rate_limiter.get_limit_status(&test_id).await {
            assert_eq!(status.attempts, 1);
            assert_eq!(status.remaining, 2);
            assert_eq!(status.is_blocked, false);
        } else {
            panic!("Failed to get rate limit status");
        }

        // Second attempt should succeed
        let result = rate_limiter.check_rate_limit(&test_id).await;
        assert!(result.is_ok(), "Second attempt should succeed");

        // Third attempt should succeed (at the limit)
        let result = rate_limiter.check_rate_limit(&test_id).await;
        assert!(result.is_ok(), "Third attempt should succeed");

        // Fourth attempt should fail and trigger blocking
        let result = rate_limiter.check_rate_limit(&test_id).await;
        assert!(result.is_err(), "Fourth attempt should fail");
        
        if let Err(AppError::RateLimitError(msg)) = result {
            assert!(msg.contains("Test rate limit exceeded"));
            assert!(msg.contains("Try again in 5 seconds"));
        } else {
            panic!("Expected RateLimitError");
        }

        // Get status after blocking
        if let Some(status) = rate_limiter.get_limit_status(&test_id).await {
            assert_eq!(status.attempts, 3);
            assert_eq!(status.remaining, 0);
            assert_eq!(status.is_blocked, true);
            assert!(status.block_reset.is_some());
        } else {
            panic!("Failed to get rate limit status");
        }

        // Wait for block to expire
        sleep(TokioDuration::from_secs(6)).await;

        // After block expires, should be able to try again
        let result = rate_limiter.check_rate_limit(&test_id).await;
        assert!(result.is_ok(), "After block expiry, attempt should succeed");

        // Check window expiry
        // Reset the counter
        let _ = rate_limiter.record_successful_attempt(&test_id, true).await;
        
        // Make two attempts
        let _ = rate_limiter.check_rate_limit(&test_id).await;
        let _ = rate_limiter.check_rate_limit(&test_id).await;
        
        // Wait for window to expire
        sleep(TokioDuration::from_secs(4)).await;
        
        // Even after two attempts, should be able to make max_attempts more
        // since the window has expired
        for i in 0..3 {
            let result = rate_limiter.check_rate_limit(&test_id).await;
            assert!(result.is_ok(), "Attempt {} after window expiry should succeed", i+1);
        }
        
        // Fourth attempt should fail again
        let result = rate_limiter.check_rate_limit(&test_id).await;
        assert!(result.is_err(), "Fourth attempt after window expiry should fail");

        // Test path-specific limits
        let path_limits = {
            let mut map = HashMap::new();
            map.insert("/api/sensitive".to_string(), 1); // Only 1 attempt allowed
            map.insert("/api/normal".to_string(), 5);    // 5 attempts allowed
            map
        };
        
        let rate_limiter = rate_limiter.with_path_limits(path_limits);
        
        // Create a new test ID for path testing
        let path_test_id = format!("test-path-{}", Uuid::new_v4());
        
        // For sensitive path, only 1 attempt should be allowed
        let result = rate_limiter.check_rate_limit_for_path(&path_test_id, "/api/sensitive").await;
        assert!(result.is_ok(), "First attempt on sensitive path should succeed");
        
        let result = rate_limiter.check_rate_limit_for_path(&path_test_id, "/api/sensitive").await;
        assert!(result.is_err(), "Second attempt on sensitive path should fail");
        
        // For normal path, 5 attempts should be allowed
        for i in 0..5 {
            let result = rate_limiter.check_rate_limit_for_path(&path_test_id, "/api/normal").await;
            assert!(result.is_ok(), "Attempt {} on normal path should succeed", i+1);
        }
        
        let result = rate_limiter.check_rate_limit_for_path(&path_test_id, "/api/normal").await;
        assert!(result.is_err(), "Sixth attempt on normal path should fail");

        // Clean up after test
        let _ = rate_limiter.record_successful_attempt(&test_id, true).await;
        let _ = rate_limiter.record_successful_attempt(&path_test_id, true).await;
        
        println!("Integration test completed successfully");
    }

    // Test for API rate limiter factory
    #[tokio::test]
    async fn test_api_rate_limiter_factory() {
        let redis_url = match env::var("REDIS_URL") {
            Ok(url) => url,
            Err(_) => {
                println!("Skipping integration test, REDIS_URL not set");
                return;
            }
        };

        let path_limits = {
            let mut map = HashMap::new();
            map.insert("/api/high_traffic".to_string(), 200); // 200 requests per minute
            map.insert("/api/low_traffic".to_string(), 50);   // 50 requests per minute
            map
        };

        let api_limiter = match create_redis_api_rate_limiter(&redis_url, Some(path_limits)).await {
            Ok(limiter) => limiter,
            Err(e) => {
                println!("Failed to create API rate limiter: {:?}", e);
                return;
            }
        };

        // Verify default limit is 100
        assert_eq!(api_limiter.config.max_attempts, 100);
        
        // Verify path-specific limits
        assert_eq!(api_limiter.get_limit_for_path("/api/high_traffic"), 200);
        assert_eq!(api_limiter.get_limit_for_path("/api/low_traffic"), 50);
        assert_eq!(api_limiter.get_limit_for_path("/api/unknown"), 100); // Default

        // Verify no blocking for API rate limiter
        assert!(api_limiter.config.block_duration.is_none());
    }

    // Test for login rate limiter factory
    #[tokio::test]
    async fn test_login_rate_limiter_factory() {
        let redis_url = match env::var("REDIS_URL") {
            Ok(url) => url,
            Err(_) => {
                println!("Skipping integration test, REDIS_URL not set");
                return;
            }
        };

        let login_limiter = match create_redis_login_rate_limiter(&redis_url).await {
            Ok(limiter) => limiter,
            Err(e) => {
                println!("Failed to create login rate limiter: {:?}", e);
                return;
            }
        };

        // Verify login limiter settings
        assert_eq!(login_limiter.config.max_attempts, 5);
        assert_eq!(login_limiter.config.window_duration, Duration::from_secs(300));
        assert_eq!(
            login_limiter.config.block_duration, 
            Some(Duration::from_secs(900))
        );
        assert!(login_limiter.config.message_template.contains("login attempts"));
    }
}