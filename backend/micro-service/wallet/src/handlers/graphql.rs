use app_error::AppResult;
use app_middleware::JwtService;
use async_graphql::http::GraphiQLSource;
use async_graphql_axum::{GraphQLRequest, GraphQLResponse};
use axum::{
    extract::Extension,
    http::{HeaderMap, header},
    response::{Html, IntoResponse},
};
use std::sync::Arc;

use crate::service::WalletService;

// Handler for GraphQL POST requests with authentication
pub async fn graphql_handler(
    schema: Extension<crate::schema::ApiSchema>,
    jwt_service: Extension<Arc<JwtService>>,
    wallet_service: Extension<Arc<WalletService>>,
    headers: HeaderMap,
    req: GraphQLRequest,
) -> AppResult<GraphQLResponse> {
    // Create a new request builder for modifying the GraphQL request
    let mut req_builder = req.into_inner();

    // IMPORTANT: Add wallet service to the request context
    req_builder = req_builder.data(Arc::clone(&wallet_service));

    // Check for authorization header
    if let Some(auth_header) = headers.get(header::AUTHORIZATION) {
        if let Ok(auth_str) = auth_header.to_str() {
            if auth_str.starts_with("Bearer ") {
                let token = &auth_str["Bearer ".len()..];

                // Validate the token
                if let Ok(claims) = jwt_service.validate_token(token) {
                    // Add the claims to the request data
                    req_builder = req_builder.data(claims);
                }
            }
        }
    }

    // Execute the GraphQL request
    let response = schema.execute(req_builder).await;

    Ok(response.into())
}

// Handler for GraphQL playground UI
pub async fn graphql_playground() -> impl IntoResponse {
    Html(GraphiQLSource::build().endpoint("/graphql").finish())
}

// Simple health check endpoint
pub async fn health_check() -> impl IntoResponse {
    (
        axum::http::StatusCode::OK,
        Html(
            "<html>
                <head>
                    <title>Wallet Service Health Check</title>
                    <style>
                        body {
                            font-family: Arial, sans-serif;
                            background-color: #f4f4f9;
                            color: #333;
                            text-align: center;
                            padding: 50px;
                        }
                        h1 {
                            color: green;
                        }
                        p {
                            font-size: 18px;
                        }
                    </style>
                </head>
                <body>
                    <h1>Wallet Service Health Check</h1>
                    <p>Status: <strong>OK</strong></p>
                    <p>The wallet service is up and running smoothly.</p>
                </body>
            </html>",
        ),
    )
}
