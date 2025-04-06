use app_database::{DB_ARC, db_connect::initialize_memory_db, service::DbService};
use app_error::AppResult;
use app_middleware::{
    create_redis_login_rate_limiter,
    limits::rate_limiter::{RateLimitConfig, RedisRateLimiter},
};
use app_models::user::User;
use axum::{
    body::Body,
    extract::Request,
    http::{Method, StatusCode},
};
use micro_user::{routes::create_routes, schema::create_schema, service::AuthService};
use serde_json::{Value, json};
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
    time::Duration,
};
use tower::ServiceExt;

// Mock Redis rate limiter for system testing
#[derive(Clone)]
struct MockRateLimiter {
    // Track login attempts by username
    login_attempts: Arc<Mutex<HashMap<String, usize>>>,
    // Track blocked status
    blocked_users: Arc<Mutex<HashMap<String, bool>>>,
    // Configuration
    max_attempts: usize,
    block_duration: Option<Duration>,
}

impl MockRateLimiter {
    fn new(max_attempts: usize, block_duration: Option<Duration>) -> Self {
        Self {
            login_attempts: Arc::new(Mutex::new(HashMap::new())),
            blocked_users: Arc::new(Mutex::new(HashMap::new())),
            max_attempts,
            block_duration,
        }
    }

    async fn record_failed_attempt(&self, identifier: &str) -> AppResult<()> {
        let mut attempts = self.login_attempts.lock().unwrap();
        let count = attempts.entry(identifier.to_string()).or_insert(0);
        *count += 1;

        // If exceeded, block the user
        if *count >= self.max_attempts && self.block_duration.is_some() {
            let mut blocked = self.blocked_users.lock().unwrap();
            blocked.insert(identifier.to_string(), true);
        }

        Ok(())
    }

    // For testing: directly block a user
    fn block_user(&self, identifier: &str) {
        let mut blocked = self.blocked_users.lock().unwrap();
        blocked.insert(identifier.to_string(), true);
    }

    // For testing: directly unblock a user
    fn unblock_user(&self, identifier: &str) {
        let mut blocked = self.blocked_users.lock().unwrap();
        blocked.remove(identifier);
    }

    // For testing: get attempt count
    fn get_attempt_count(&self, identifier: &str) -> usize {
        let attempts = self.login_attempts.lock().unwrap();
        *attempts.get(identifier).unwrap_or(&0)
    }
}

// Helper function to create a test app with configurable rate limiter and JWT settings
async fn setup_test_app_with_config(
    max_login_attempts: usize,
    block_duration: Option<Duration>,
    jwt_expiry_hours: u64,
) -> AppResult<(axum::Router, Arc<MockRateLimiter>, Arc<AuthService>)> {
    // Setup in-memory database
    let db_arc = DB_ARC
        .get_or_init(|| async {
            initialize_memory_db().await.unwrap_or_else(|_e| {
                panic!("Database initialization failed");
            })
        })
        .await;
    let user_db = Arc::new(DbService::<User>::new(db_arc, "users"));

    // Create JWT service with configurable expiry
    let jwt_secret = b"test_secret_key_for_system_testing_only";
    let mock_rate_limiter = Arc::new(MockRateLimiter::new(max_login_attempts, block_duration));

    let login_rate_limiter = Arc::new(
        create_redis_login_rate_limiter(&"redis://:redis_secure_password@0.0.0.0:6379").await?,
    );

    // Create auth service with test configuration
    let auth_service = Arc::new(
        AuthService::new(jwt_secret, jwt_expiry_hours)
            .with_db(user_db)
            .with_rate_limiter(login_rate_limiter),
    );

    // Create schema and routes
    let schema = create_schema();
    let app = create_routes(
        schema,
        Arc::clone(&auth_service),
        Arc::new(
            RedisRateLimiter::new(
                "redis://:redis_secure_password@0.0.0.0:6379",
                RateLimitConfig {
                    max_attempts: 100,
                    window_duration: Duration::from_secs(60),
                    block_duration: None,
                    message_template: "Test rate limit".into(),
                },
            )
            .await
            .unwrap(),
        ),
    );

    Ok((app, mock_rate_limiter, auth_service))
}

// Helper to make GraphQL requests
async fn graphql_request(
    app: &axum::Router,
    query: &str,
    variables: Option<Value>,
    auth_token: Option<&str>,
) -> (StatusCode, Value) {
    // Build the request JSON
    let mut request_json = json!({
        "query": query
    });

    if let Some(vars) = variables {
        request_json["variables"] = vars;
    }

    // Convert the JSON to a string
    let body_string = serde_json::to_string(&request_json).unwrap();

    // Create the request
    let mut req_builder = Request::builder()
        .uri("/graphql")
        .method(Method::POST)
        .header("Content-Type", "application/json");

    // Add auth token if provided
    if let Some(token) = auth_token {
        req_builder = req_builder.header("Authorization", format!("Bearer {}", token));
    }

    let request = req_builder.body(Body::from(body_string)).unwrap();

    // Send the request
    let response = app.clone().oneshot(request).await.unwrap();

    // Extract the status and body
    let status = response.status();
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();

    // Parse the body as JSON
    let body_json: Value = serde_json::from_slice(&body).unwrap_or_else(|_| json!({}));

    (status, body_json)
}

// Test 1: Rate limiting functionality
#[tokio::test]
#[ignore]
async fn test_rate_limiting() -> AppResult<()> {
    // Setup app with 3 max attempts and 5 second block duration
    let (app, rate_limiter, _auth_service) =
        setup_test_app_with_config(3, Some(Duration::from_secs(5)), 24).await?;

    // Register a test user first
    let register_query = r#"
    mutation Register($input: RegisterInput!) {
        register(input: $input) {
            token
            user {
                id
                username
            }
        }
    }
    "#;

    let username = format!("ratelimit_{}", chrono::Utc::now().timestamp());
    let register_vars = json!({
        "input": {
            "name": "Rate Limit Test",
            "username": username,
            "email": format!("{}@example.com", username),
            "password": "RateLimit@123"
        }
    });

    let (status, body) = graphql_request(&app, register_query, Some(register_vars), None).await;
    assert_eq!(status, StatusCode::OK);
    assert!(
        !body["data"]["register"]["token"]
            .as_str()
            .unwrap()
            .is_empty()
    );

    // Now try to login with wrong password multiple times
    let login_query = r#"
    mutation Login($input: LoginInput!) {
        login(input: $input) {
            token
        }
    }
    "#;

    let login_vars = json!({
        "input": {
            "username": username,
            "password": "WrongPassword@123"
        }
    });

    // First attempt - should fail but not rate limit
    let (_, body) = graphql_request(&app, login_query, Some(login_vars.clone()), None).await;
    assert!(body["errors"].is_array());
    assert!(
        body["errors"][0]["message"]
            .as_str()
            .unwrap()
            .contains("Login failed")
    );

    // Force rate limiter to record the failed attempt
    rate_limiter.record_failed_attempt(&username).await?;

    // Second attempt - should fail but not rate limit
    let (_, body) = graphql_request(&app, login_query, Some(login_vars.clone()), None).await;
    assert!(body["errors"].is_array());
    assert!(
        body["errors"][0]["message"]
            .as_str()
            .unwrap()
            .contains("Login failed")
    );

    // Force rate limiter to record the failed attempt
    rate_limiter.record_failed_attempt(&username).await?;

    // Third attempt - should fail but not rate limit
    let (_, body) = graphql_request(&app, login_query, Some(login_vars.clone()), None).await;
    assert!(body["errors"].is_array());
    assert!(
        body["errors"][0]["message"]
            .as_str()
            .unwrap()
            .contains("Login failed")
    );

    // Force rate limiter to record the failed attempt - this should trigger blocking
    rate_limiter.record_failed_attempt(&username).await?;

    // Manually verify the user is now blocked
    assert_eq!(rate_limiter.get_attempt_count(&username), 3);
    rate_limiter.block_user(&username);

    // Fourth attempt - should be rate limited
    let (_, body) = graphql_request(&app, login_query, Some(login_vars.clone()), None).await;
    assert!(body["errors"].is_array());
    assert!(
        body["errors"][0]["message"]
            .as_str()
            .unwrap()
            .contains("Rate limit error")
    );

    // Unblock and try again
    rate_limiter.unblock_user(&username);
    let login_vars = json!({
        "input": {
            "username": username,
            "password": "RateLimit@123" // Correct password
        }
    });

    // Should succeed now
    let (status, _body) = graphql_request(&app, login_query, Some(login_vars), None).await;
    assert_eq!(status, StatusCode::OK);

    Ok(())
}

#[tokio::test]
#[ignore]
async fn test_error_responses() -> AppResult<()> {
    // Setup app with standard config
    let (app, _, _) = setup_test_app_with_config(5, None, 24).await?;

    // Test Case 1: GraphQL syntax error
    let invalid_query = r#"
    query {
        invalidField {
            this is not valid GraphQL
        }
    }
    "#;

    let (status, body) = graphql_request(&app, invalid_query, None, None).await;
    assert_eq!(status, StatusCode::OK);
    assert!(body["errors"].is_array());

    // Test Case 2: Validation errors
    let register_query = r#"
    mutation Register($input: RegisterInput!) {
        register(input: $input) {
            token
        }
    }
    "#;

    // Empty username
    let empty_username_vars = json!({
        "input": {
            "name": "Test User",
            "username": "",
            "email": "test@example.com",
            "password": "Password@123"
        }
    });

    let (_, body) = graphql_request(&app, register_query, Some(empty_username_vars), None).await;
    assert!(body["errors"].is_array());
    assert!(
        body["errors"][0]["message"]
            .as_str()
            .unwrap()
            .contains("Username cannot be empty")
    );

    // Invalid email
    let invalid_email_vars = json!({
        "input": {
            "name": "Test User",
            "username": "testuser",
            "email": "not-an-email",
            "password": "Password@123"
        }
    });

    let (_, body) = graphql_request(&app, register_query, Some(invalid_email_vars), None).await;
    assert!(body["errors"].is_array());
    assert!(
        body["errors"][0]["message"]
            .as_str()
            .unwrap()
            .contains("Invalid email format")
    );

    // Weak password
    let weak_password_vars = json!({
        "input": {
            "name": "Test User",
            "username": "testuser",
            "email": "test@example.com",
            "password": "weak"
        }
    });

    let (_, body) = graphql_request(&app, register_query, Some(weak_password_vars), None).await;
    assert!(body["errors"].is_array());
    assert!(
        body["errors"][0]["message"]
            .as_str()
            .unwrap()
            .contains("Password must")
    );

    // Test Case 3: Authentication error
    let me_query = r#"
    query {
        me {
            id
            username
        }
    }
    "#;

    // Invalid token
    let invalid_token = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiIxMjM0NTY3ODkwIiwibmFtZSI6IkpvaG4gRG9lIiwiaWF0IjoxNTE2MjM5MDIyfQ.SflKxwRJSMeKKF2QT4fwpMeJf36POk6yJV_adQssw5c";

    let (_, body) = graphql_request(&app, me_query, None, Some(invalid_token)).await;
    assert!(body["errors"].is_array());
    assert!(
        body["errors"][0]["message"]
            .as_str()
            .unwrap()
            .contains("Authentication required")
    );

    Ok(())
}

// Test 4: Database edge cases
#[ignore]
#[tokio::test]
async fn test_database_edge_cases() -> AppResult<()> {
    // Setup app with standard config
    let (app, _, _) = setup_test_app_with_config(5, None, 24).await?;

    // Test Case 1: Duplicate username
    let register_query = r#"
    mutation Register($input: RegisterInput!) {
        register(input: $input) {
            token
        }
    }
    "#;

    let username = format!("dbtest_{}", chrono::Utc::now().timestamp());
    let register_vars = json!({
        "input": {
            "name": "DB Test",
            "username": username,
            "email": format!("{}@example.com", username),
            "password": "DBTest@123"
        }
    });

    // First registration should succeed
    let (status, _) =
        graphql_request(&app, register_query, Some(register_vars.clone()), None).await;
    assert_eq!(status, StatusCode::OK);

    // Second registration with same username should fail
    let (_, body) = graphql_request(&app, register_query, Some(register_vars), None).await;
    assert!(body["errors"].is_array());
    assert!(
        body["errors"][0]["message"]
            .as_str()
            .unwrap()
            .contains("already registered")
    );

    // Test Case 2: Duplicate email
    let new_username = format!("dbtest2_{}", chrono::Utc::now().timestamp());
    let email = format!("{}@example.com", username); // Same email as before
    let duplicate_email_vars = json!({
        "input": {
            "name": "DB Test 2",
            "username": new_username,
            "email": email,
            "password": "DBTest@123"
        }
    });

    let (_, body) = graphql_request(&app, register_query, Some(duplicate_email_vars), None).await;
    assert!(body["errors"].is_array());
    assert!(
        body["errors"][0]["message"]
            .as_str()
            .unwrap()
            .contains("Email already registered")
    );

    // Test Case 3: Non-existent user login
    let login_query = r#"
    mutation Login($input: LoginInput!) {
        login(input: $input) {
            token
        }
    }
    "#;

    let nonexistent_vars = json!({
        "input": {
            "username": "nonexistentuser",
            "password": "Whatever@123"
        }
    });

    let (_, body) = graphql_request(&app, login_query, Some(nonexistent_vars), None).await;
    assert!(body["errors"].is_array());
    assert!(
        body["errors"][0]["message"]
            .as_str()
            .unwrap()
            .contains("Login failed")
    );

    Ok(())
}
