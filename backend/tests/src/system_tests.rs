use app_database::{DB_ARC, db_connect::initialize_memory_db, service::DbService};
use app_error::AppResult;
use app_middleware::limits::rate_limiter::{RateLimitConfig, RedisRateLimiter};
use app_models::user::User;
use axum::{
    body::Body,
    extract::Request,
    http::{Method, StatusCode},
};
use micro_user::{routes::create_routes, schema::create_schema, service::AuthService};
use serde_json::{Value, json};
use std::{sync::Arc, time::Duration};
use tower::ServiceExt;

// Helper function to create a test app instance
async fn setup_test_app() -> AppResult<axum::Router> {
    // Setup in-memory database

    let db_arc = DB_ARC
        .get_or_init(|| async {
            initialize_memory_db().await.unwrap_or_else(|_e| {
                panic!("Database initialization failed");
            })
        })
        .await;

    let user_db = Arc::new(DbService::<User>::new(db_arc, "users"));

    // Create JWT service with a test secret
    let jwt_secret = b"test_secret_key_for_system_testing_only";
    let expiry_hours = 24; // longer for tests
    let auth_service = Arc::new(AuthService::new(jwt_secret, expiry_hours).with_db(user_db));

    // Create mock rate limiter
    let api_rate_limiter = Arc::new(
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
    );

    // Create schema and routes
    let schema = create_schema();
    let app = create_routes(schema, auth_service, api_rate_limiter);

    Ok(app)
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

#[ignore]
#[tokio::test]
async fn test_health_check_endpoint() -> AppResult<()> {
    // Setup app
    let app = setup_test_app().await?;

    // Make request to health endpoint
    let request = Request::builder()
        .uri("/health")
        .method(Method::GET)
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    // Verify response
    assert_eq!(response.status(), StatusCode::OK);

    Ok(())
}

#[ignore]
#[tokio::test]
async fn test_register_login_flow() -> AppResult<()> {
    // Setup app
    let app = setup_test_app().await?;

    // 1. Register a new user
    let register_query = r#"
            mutation Register($input: RegisterInput!) {
                register(input: $input) {
                    token
                    user {
                        id
                        name
                        username
                        email
                        address
                    }
                }
            }
            "#;

    let register_vars = json!({
        "input": {
            "name": "Test User",
            "username": "testuser123",
            "email": "test@example.com",
            "password": "Test@123456"
        }
    });

    let (status, body) = graphql_request(&app, register_query, Some(register_vars), None).await;

    assert_eq!(status, StatusCode::OK);
    assert!(body["data"]["register"]["token"].is_string());
    assert_eq!(body["data"]["register"]["user"]["name"], "Test User");
    assert_eq!(body["data"]["register"]["user"]["username"], "testuser123");
    assert_eq!(
        body["data"]["register"]["user"]["email"],
        "test@example.com"
    );

    // Extract token for next steps
    let token = body["data"]["register"]["token"]
        .as_str()
        .unwrap()
        .to_string();

    // 2. Log in with the same user
    let login_query = r#"
            mutation Login($input: LoginInput!) {
                login(input: $input) {
                    token
                    user {
                        id
                        name
                        username
                        email
                    }
                }
            }
            "#;

    let login_vars = json!({
        "input": {
            "username": "testuser123",
            "password": "Test@123456"
        }
    });

    let (status, body) = graphql_request(&app, login_query, Some(login_vars), None).await;

    assert_eq!(status, StatusCode::OK);
    assert!(body["data"]["login"]["token"].is_string());
    assert_eq!(body["data"]["login"]["user"]["name"], "Test User");
    assert_eq!(body["data"]["login"]["user"]["username"], "testuser123");

    // 3. Get current user with token
    let me_query = r#"
            query {
                me {
                    id
                    name
                    username
                    email
                }
            }
        "#;

    let (status, body) = graphql_request(&app, me_query, None, Some(&token)).await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["data"]["me"]["name"], "Test User");
    assert_eq!(body["data"]["me"]["username"], "testuser123");
    assert_eq!(body["data"]["me"]["email"], "test@example.com");

    Ok(())
}

#[ignore]
#[tokio::test]
async fn test_password_validation() -> AppResult<()> {
    // Setup app
    let app = setup_test_app().await?;

    // Try to register with weak password
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

    let register_vars = json!({
        "input": {
            "name": "Test User",
            "username": "testuser456",
            "email": "test2@example.com",
            "password": "weak"  // Too short, missing required characters
        }
    });

    let (status, body) = graphql_request(&app, register_query, Some(register_vars), None).await;

    assert_eq!(status, StatusCode::OK);
    assert!(body["errors"].is_array());
    // Verify error contains password validation message
    let error_message = body["errors"][0]["message"].as_str().unwrap();
    assert!(error_message.contains("Password must"));

    Ok(())
}

#[ignore]
#[tokio::test]
async fn test_duplicate_username() -> AppResult<()> {
    // Setup app
    let app = setup_test_app().await?;

    // Register first user
    let register_query = r#"
            mutation Register($input: RegisterInput!) {
                register(input: $input) {
                    token
                }
            }
            "#;

    let register_vars = json!({
        "input": {
            "name": "Original User",
            "username": "duplicate",
            "email": "original@example.com",
            "password": "Original@123"
        }
    });

    let (status, _) = graphql_request(&app, register_query, Some(register_vars), None).await;
    assert_eq!(status, StatusCode::OK);

    // Try to register with same username
    let duplicate_vars = json!({
        "input": {
            "name": "Duplicate User",
            "username": "duplicate", // Same username
            "email": "different@example.com",
            "password": "Different@123"
        }
    });

    let (status, body) = graphql_request(&app, register_query, Some(duplicate_vars), None).await;

    assert_eq!(status, StatusCode::OK);
    assert!(body["errors"].is_array());
    // Verify error contains duplicate username message
    let error_message = body["errors"][0]["message"].as_str().unwrap();
    assert!(error_message.contains("already registered"));

    Ok(())
}

#[ignore]
#[tokio::test]
async fn test_unauthenticated_me_query() -> AppResult<()> {
    // Setup app
    let app = setup_test_app().await?;

    // Try to access me query without authentication
    let me_query = r#"
            query {
                me {
                    id
                    name
                    username
                }
            }
            "#;

    let (status, body) = graphql_request(&app, me_query, None, None).await;

    assert_eq!(status, StatusCode::OK);
    assert!(body["errors"].is_array());
    // Verify error contains authentication required message
    let error_message = body["errors"][0]["message"].as_str().unwrap();
    assert!(error_message.contains("Authentication required"));

    Ok(())
}

#[ignore]
#[tokio::test]
async fn test_invalid_login() -> AppResult<()> {
    // Setup app
    let app = setup_test_app().await?;

    // Register a user first
    let register_query = r#"
            mutation Register($input: RegisterInput!) {
                register(input: $input) {
                    token
                }
            }
            "#;

    let register_vars = json!({
        "input": {
            "name": "Login Test User",
            "username": "logintest",
            "email": "login@example.com",
            "password": "Correct@123"
        }
    });

    let (status, _) = graphql_request(&app, register_query, Some(register_vars), None).await;
    assert_eq!(status, StatusCode::OK);

    // Try login with wrong password
    let login_query = r#"
            mutation Login($input: LoginInput!) {
                login(input: $input) {
                    token
                }
            }
            "#;

    let login_vars = json!({
        "input": {
            "username": "logintest",
            "password": "Wrong@123"
        }
    });

    let (status, body) = graphql_request(&app, login_query, Some(login_vars), None).await;

    assert_eq!(status, StatusCode::OK);
    assert!(body["errors"].is_array());
    // Verify error contains login failed message
    let error_message = body["errors"][0]["message"].as_str().unwrap();
    assert!(error_message.contains("Login failed"));

    // Try login with non-existent user
    let nonexistent_vars = json!({
        "input": {
            "username": "doesnotexist",
            "password": "Whatever@123"
        }
    });

    let (status, body) = graphql_request(&app, login_query, Some(nonexistent_vars), None).await;

    assert_eq!(status, StatusCode::OK);
    assert!(body["errors"].is_array());
    // Verify error contains login failed message
    let error_message = body["errors"][0]["message"].as_str().unwrap();
    assert!(error_message.contains("Login failed"));

    Ok(())
}

#[ignore]
#[tokio::test]
async fn test_input_validation() -> AppResult<()> {
    // Setup app
    let app = setup_test_app().await?;

    // Test cases for validation
    let test_cases = vec![
        // Empty username
        (
            json!({
                "input": {
                    "name": "Valid Name",
                    "username": "",
                    "email": "valid@example.com",
                    "password": "Valid@123"
                }
            }),
            "Username cannot be empty",
        ),
        // Invalid email format
        (
            json!({
                "input": {
                    "name": "Valid Name",
                    "username": "validuser",
                    "email": "not-an-email",
                    "password": "Valid@123"
                }
            }),
            "Invalid email format",
        ),
        // Name too short
        (
            json!({
                "input": {
                    "name": "A",
                    "username": "validuser",
                    "email": "valid@example.com",
                    "password": "Valid@123"
                }
            }),
            "Name must be at least",
        ),
    ];

    let register_query = r#"
        mutation Register($input: RegisterInput!) {
            register(input: $input) {
                token
            }
        }
        "#;

    for (variables, expected_error) in test_cases {
        let (status, body) = graphql_request(&app, register_query, Some(variables), None).await;

        assert_eq!(status, StatusCode::OK);
        assert!(body["errors"].is_array());
        // Verify error contains expected validation message
        let error_message = body["errors"][0]["message"].as_str().unwrap();
        assert!(
            error_message.contains(expected_error),
            "Expected error to contain '{}', but got: '{}'",
            expected_error,
            error_message
        );
    }

    Ok(())
}
