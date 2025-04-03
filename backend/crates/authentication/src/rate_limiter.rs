use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use app_error::{AppError, AppResult};
use tokio::sync::RwLock;

/// Structure to track login attempts
#[derive(Debug)]
struct LoginAttempt {
    attempts: usize,
    first_attempt: Instant,
    last_attempt: Instant,
}

/// Rate limiter for login attempts
#[derive(Debug, Clone)]
pub struct LoginRateLimiter {
    attempts: Arc<RwLock<HashMap<String, LoginAttempt>>>,
    max_attempts: usize,
    window_duration: Duration,
    lockout_duration: Duration,
}

impl LoginRateLimiter {
    /// Create a new rate limiter
    pub fn new(max_attempts: usize, window_duration: Duration, lockout_duration: Duration) -> Self {
        Self {
            attempts: Arc::new(RwLock::new(HashMap::new())),
            max_attempts,
            window_duration,
            lockout_duration,
        }
    }

    /// Create a default rate limiter with sensible defaults
    pub fn default() -> Self {
        // Default: 5 attempts within 5 minutes, 15 minute lockout
        Self::new(
            5,
            Duration::from_secs(5 * 60),
            Duration::from_secs(15 * 60),
        )
    }

    /// Check if a user can make a login attempt
    pub async fn check_rate_limit(&self, identifier: &str) -> AppResult<()> {
        let mut attempts = self.attempts.write().await;
        let now = Instant::now();
        
        // Clean up old entries
        self.cleanup(&mut attempts, now);
        
        // Check if the user is in the map
        if let Some(attempt) = attempts.get(identifier) {
            // If user has exceeded max attempts within window, check if lockout period has passed
            if attempt.attempts >= self.max_attempts {
                let elapsed_since_last = now.duration_since(attempt.last_attempt);
                
                // If still in lockout period, reject
                if elapsed_since_last < self.lockout_duration {
                    let seconds_remaining = (self.lockout_duration - elapsed_since_last).as_secs();
                    return Err(AppError::AuthenticationError(
                        format!("Too many login attempts. Please try again in {} seconds", seconds_remaining)
                    ));
                }
                
                // Lockout period passed, remove the entry
                attempts.remove(identifier);
            }
        }
        
        Ok(())
    }
    
    /// Record a failed login attempt
    pub async fn record_failed_attempt(&self, identifier: &str) {
        let mut attempts = self.attempts.write().await;
        let now = Instant::now();
        
        match attempts.get_mut(identifier) {
            Some(attempt) => {
                // Update existing record
                attempt.attempts += 1;
                attempt.last_attempt = now;
            }
            None => {
                // Create new record
                attempts.insert(
                    identifier.to_string(),
                    LoginAttempt {
                        attempts: 1,
                        first_attempt: now,
                        last_attempt: now,
                    },
                );
            }
        }
    }
    
    /// Record a successful login attempt
    pub async fn record_successful_attempt(&self, identifier: &str) {
        let mut attempts = self.attempts.write().await;
        attempts.remove(identifier);
    }
    
    /// Clean up old entries
    fn cleanup(&self, attempts: &mut HashMap<String, LoginAttempt>, now: Instant) {
        attempts.retain(|_, attempt| {
            // Keep entry if it's within the window or lockout period
            let elapsed = now.duration_since(attempt.first_attempt);
            elapsed < self.window_duration || 
                (attempt.attempts >= self.max_attempts && 
                 now.duration_since(attempt.last_attempt) < self.lockout_duration)
        });
    }
}
