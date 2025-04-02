mod tests {
    use axum::{
        body::Body,
        http::{Request, StatusCode, header},
    };
    use serde_json::{Value, json};
    use std::{env, sync::Arc};
    use tokio::test;
    use tower::ServiceExt;

    use crate::{routes::create_routes, schema::create_schema};
    use app_authentication::{AuthService, service::AuthServiceTrait};
    use app_database::{DB_ARC, db_connect::initialize_memory_db, service::DbService};
    use app_models::user::{RegisterInput, User};
    use tracing::error;

    async fn setup_test_environment() -> Arc<AuthService> {
        // Set up environment variables
        unsafe { env::set_var("DB_NAMESPACE", "test_namespace") };
        unsafe { env::set_var("DB_NAME", "test_db") };
        unsafe { env::set_var("SURREALDB_USERNAME", "test_user") };
        unsafe { env::set_var("SURREALDB_PASSWORD", "test_password") };
        unsafe { env::set_var("DB_POOL_SIZE", "5") };

        let db_arc = DB_ARC
            .get_or_init(|| async {
                initialize_memory_db().await.unwrap_or_else(|e| {
                    error!("Database initialization failed: {}", e);
                    panic!("Database initialization failed");
                })
            })
            .await;

        let user_db = Arc::new(DbService::<User>::new(db_arc, "users"));

        // Setup auth service with JWT
        let jwt_secret = "test_jwt_secret".as_bytes().to_vec();
        let auth_service = Arc::new(AuthService::new(&jwt_secret).with_db(user_db));

        auth_service
    }

    #[test]
    async fn test_graphql_health_check() {
        // Initialize the test environment
        let auth_service = setup_test_environment().await;

        // Create GraphQL schema
        let schema = create_schema();

        // Configure application routes
        let app = create_routes(schema, auth_service);

        // Create a request to the health check endpoint
        let request = Request::builder()
            .uri("/health")
            .body(Body::empty())
            .unwrap();

        // Process the request
        let response = app.oneshot(request).await.unwrap();

        // Check the response
        assert_eq!(response.status(), StatusCode::OK);
        cleanup();
    }

    #[ignore]
    #[tokio::test]
    async fn test_graphql_register_user() {
        // Initialize the test environment
        let auth_service = setup_test_environment().await;

        // Create GraphQL schema
        let schema = create_schema();

        // Configure application routes
        let app = create_routes(schema, auth_service);

        // Create a GraphQL register mutation
        let register_mutation = json!({
            "query": r#"
                mutation RegisterUser($input: RegisterInput!) {
                    register(input: $input) {
                        token
                        user {
                            id
                            username
                            name
                            email
                        }
                    }
                }
            "#,
            "variables": {
                "input": {
                    "name": "Test User",
                    "username": "testuser",
                    "email": "test@example.com",
                    "password": "Password123!"
                }
            }
        });

        // Create a request to the GraphQL endpoint
        let request = Request::builder()
            .uri("/graphql")
            .header(header::CONTENT_TYPE, "application/json")
            .method("POST")
            .body(Body::from(
                serde_json::to_string(&register_mutation).unwrap(),
            ))
            .unwrap();

        // Process the request
        let response = app.clone().oneshot(request).await.unwrap();

        // Check the response status
        assert_eq!(response.status(), StatusCode::OK);

        // Parse the response body
        let body = axum::body::to_bytes(response.into_body(), 1024 * 1024)
            .await
            .unwrap(); // Set limit to 1MB
        let json_response: Value = serde_json::from_slice(&body).unwrap();

        // Verify the response data structure
        let data = &json_response["data"]["register"];
        assert!(data["token"].is_string(), "Response should include a token");
        assert_eq!(
            data["user"]["username"], "testuser",
            "Username should match input"
        );
        assert_eq!(data["user"]["name"], "Test User", "Name should match input");
        assert_eq!(
            data["user"]["email"], "test@example.com",
            "Email should match input"
        );
        cleanup();
    }

    #[ignore]
    #[tokio::test]
    async fn test_graphql_login_user() {
        // Initialize the test environment
        let auth_service = setup_test_environment().await;

        // Create GraphQL schema
        let schema = create_schema();

        // Configure application routes
        let app = create_routes(schema, auth_service.clone());

        // First register a user directly through the auth service
        let register_input = RegisterInput {
            name: "Login Test User".to_string(),
            username: "logintest".to_string(),
            email: "logintest@example.com".to_string(),
            password: "Password123!".to_string(),
        };

        auth_service
            .register(register_input)
            .await
            .expect("Failed to register test user");

        // Create a GraphQL login mutation
        let login_mutation = json!({
            "query": r#"
                mutation LoginUser($input: LoginInput!) {
                    login(input: $input) {
                        token
                        user {
                            id
                            username
                            name
                            email
                        }
                    }
                }
            "#,
            "variables": {
                "input": {
                    "username": "logintest",
                    "password": "Password123!"
                }
            }
        });

        // Create a request to the GraphQL endpoint
        let request = Request::builder()
            .uri("/graphql")
            .header(header::CONTENT_TYPE, "application/json")
            .method("POST")
            .body(Body::from(serde_json::to_string(&login_mutation).unwrap()))
            .unwrap();

        // Process the request
        let response = app.oneshot(request).await.unwrap();

        // Check the response status
        assert_eq!(response.status(), StatusCode::OK);

        // Parse the response body
        let body = axum::body::to_bytes(response.into_body(), 1024 * 1024)
            .await
            .unwrap(); // Set limit to 1MB
        let json_response: Value = serde_json::from_slice(&body).unwrap();

        // Verify the response data structure
        let data = &json_response["data"]["login"];
        assert!(data["token"].is_string(), "Response should include a token");
        assert_eq!(
            data["user"]["username"], "logintest",
            "Username should match input"
        );
        assert_eq!(
            data["user"]["name"], "Login Test User",
            "Name should match registered user"
        );
        assert_eq!(
            data["user"]["email"], "logintest@example.com",
            "Email should match registered user"
        );

        cleanup();
    }

    #[ignore]
    #[tokio::test]
    async fn test_graphql_authenticated_me_query() {
        // Initialize the test environment
        let auth_service = setup_test_environment().await;
        let _jwt_service = auth_service.get_jwt_service();

        // Create GraphQL schema
        let schema = create_schema();

        // Configure application routes
        let app = create_routes(schema, auth_service.clone());

        // First register a user directly through the auth service
        let register_input = RegisterInput {
            name: "Me Query User".to_string(),
            username: "mequery".to_string(),
            email: "mequery@example.com".to_string(),
            password: "Password123!".to_string(),
        };

        let auth_response = auth_service
            .register(register_input)
            .await
            .expect("Failed to register test user");
        let token = auth_response.token;

        // Create a GraphQL me query
        let me_query = json!({
            "query": r#"
                query {
                    me {
                        id
                        username
                        name
                        email
                    }
                }
            "#
        });

        // Create a request to the GraphQL endpoint with authentication
        let request = Request::builder()
            .uri("/graphql")
            .header(header::CONTENT_TYPE, "application/json")
            .header(header::AUTHORIZATION, format!("Bearer {}", token))
            .method("POST")
            .body(Body::from(serde_json::to_string(&me_query).unwrap()))
            .unwrap();

        // Process the request
        let response = app.oneshot(request).await.unwrap();

        // Check the response status
        assert_eq!(response.status(), StatusCode::OK);

        // Parse the response body
        let body = axum::body::to_bytes(response.into_body(), 1024)
            .await
            .unwrap(); // Set limit to 0.1MB
        let json_response: Value = serde_json::from_slice(&body).unwrap();

        // Verify the response data structure
        let data = &json_response["data"]["me"];
        assert_eq!(
            data["username"], "mequery",
            "Username should match registered user"
        );
        assert_eq!(
            data["name"], "Me Query User",
            "Name should match registered user"
        );
        assert_eq!(
            data["email"], "mequery@example.com",
            "Email should match registered user"
        );

        cleanup();
    }

    #[test]
    async fn test_unauthenticated_me_query() {
        // Initialize the test environment
        let auth_service = setup_test_environment().await;

        // Create GraphQL schema
        let schema = create_schema();

        // Configure application routes
        let app = create_routes(schema, auth_service);

        // Create a GraphQL me query
        let me_query = json!({
            "query": r#"
                query {
                    me {
                        id
                        username
                        name
                        email
                    }
                }
            "#
        });

        // Create a request to the GraphQL endpoint WITHOUT authentication
        let request = Request::builder()
            .uri("/graphql")
            .header(header::CONTENT_TYPE, "application/json")
            .method("POST")
            .body(Body::from(serde_json::to_string(&me_query).unwrap()))
            .unwrap();

        // Process the request
        let response = app.oneshot(request).await.unwrap();

        // Check the response status - should still be 200 OK (GraphQL returns errors in the response body)
        assert_eq!(response.status(), StatusCode::OK);

        // Parse the response body
        let body = axum::body::to_bytes(response.into_body(), 1024)
            .await
            .unwrap();
        let json_response: Value = serde_json::from_slice(&body).unwrap();

        // Verify there's an error in the response
        assert!(
            json_response["errors"].is_array(),
            "Response should include errors"
        );
        assert!(
            json_response["errors"][0]["message"]
                .as_str()
                .unwrap()
                .contains("Not authenticated"),
            "Error should indicate authentication failure"
        );

        cleanup();
    }

    #[test]
    async fn test_invalid_token_me_query() {
        // Initialize the test environment
        let auth_service = setup_test_environment().await;

        // Create GraphQL schema
        let schema = create_schema();

        // Configure application routes
        let app = create_routes(schema, auth_service);

        // Create a GraphQL me query
        let me_query = json!({
            "query": r#"
                query {
                    me {
                        id
                        username
                        name
                        email
                    }
                }
            "#
        });

        // Create a request to the GraphQL endpoint with an invalid token
        let request = Request::builder()
            .uri("/graphql")
            .header(header::CONTENT_TYPE, "application/json")
            .header(header::AUTHORIZATION, "Bearer invalid.token.here")
            .method("POST")
            .body(Body::from(serde_json::to_string(&me_query).unwrap()))
            .unwrap();

        // Process the request
        let response = app.oneshot(request).await.unwrap();

        // Check the response status - should still be 200 OK (GraphQL returns errors in the response body)
        assert_eq!(response.status(), StatusCode::OK);

        // Parse the response body
        let body = axum::body::to_bytes(response.into_body(), 1024)
            .await
            .unwrap();
        let json_response: Value = serde_json::from_slice(&body).unwrap();

        // Verify there's an error in the response
        assert!(
            json_response["errors"].is_array(),
            "Response should include errors"
        );
        assert!(
            json_response["errors"][0]["message"]
                .as_str()
                .unwrap()
                .contains("Not authenticated"),
            "Error should indicate authentication failure"
        );

        cleanup();
    }

    // Clean up function to reset environment after tests
    fn cleanup() {
        unsafe { env::remove_var("DB_ENDPOINT") };
        unsafe { env::remove_var("DB_NAMESPACE") };
        unsafe { env::remove_var("DB_NAME") };
        unsafe { env::remove_var("SURREALDB_USERNAME") };
        unsafe { env::remove_var("SURREALDB_PASSWORD") };
        unsafe { env::remove_var("DB_POOL_SIZE") };
    }
}
