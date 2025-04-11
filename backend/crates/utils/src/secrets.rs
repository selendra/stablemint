use app_error::{AppError, AppResult};
use reqwest::{Client, header};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, error, info};

/// Client for interacting with HCP Secrets API
#[derive(Clone)]
pub struct SecretsClient {
    client: Client,
    base_url: String,
    token: Arc<RwLock<Option<String>>>,
    org_id: String,
    project_id: String,
    app_name: String,
    client_id: String,
    client_secret: String,
}

#[derive(Debug, Serialize)]
struct AuthRequest {
    audience: String,
    grant_type: String,
    client_id: String,
    client_secret: String,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
struct AuthResponse {
    access_token: String,
    token_type: String,
    expires_in: u64,
}

#[derive(Debug, Serialize)]
struct SecretUpsertRequest {
    value: String,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
struct SecretResponse {
    name: String,
    version: u64,
    created_at: String,
    value: String,
}

impl SecretsClient {
    /// Create a new HCP Secrets client
    pub fn new(
        base_url: &str,
        org_id: &str,
        project_id: &str,
        app_name: &str,
        client_id: &str,
        client_secret: &str,
    ) -> Self {
        Self {
            client: Client::new(),
            base_url: base_url.to_string(),
            token: Arc::new(RwLock::new(None)),
            org_id: org_id.to_string(),
            project_id: project_id.to_string(),
            app_name: app_name.to_string(),
            client_id: client_id.to_string(),
            client_secret: client_secret.to_string(),
        }
    }

    /// Initialize and authenticate with HCP Secrets
    pub async fn initialize(&self) -> AppResult<()> {
        // Authenticate with HCP
        self.authenticate().await
    }

    /// Authenticate with HCP using client credentials
    async fn authenticate(&self) -> AppResult<()> {
        info!("Authenticating with HCP Secrets...");
        
        let auth_request = AuthRequest {
            audience: "https://api.hashicorp.cloud".to_string(),
            grant_type: "client_credentials".to_string(),
            client_id: self.client_id.clone(),
            client_secret: self.client_secret.clone(),
        };

        let auth_url = "https://auth.idp.hashicorp.com/oauth2/token";
        let response = self.client
            .post(auth_url)
            .header(header::CONTENT_TYPE, "application/json")
            .json(&auth_request)
            .send()
            .await
            .map_err(|e| {
                error!("Failed to authenticate with HCP: {}", e);
                AppError::NetworkError(format!("Failed to connect to HCP: {}", e))
            })?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            error!("HCP authentication failed with status {}: {}", status, text);
            return Err(AppError::AuthenticationError(
                format!("Failed to authenticate with HCP: HTTP {}: {}", status, text)
            ));
        }

        let auth_response: AuthResponse = response.json().await.map_err(|e| {
            error!("Failed to parse HCP authentication response: {}", e);
            AppError::NetworkError(format!("Invalid HCP response: {}", e))
        })?;

        // Store the token
        let mut token_guard = self.token.write().await;
        *token_guard = Some(auth_response.access_token);
        drop(token_guard);

        info!("Successfully authenticated with HCP Secrets");
        Ok(())
    }

    /// Get secret from HCP Secrets
    pub async fn get_secret(&self, secret_name: &str) -> AppResult<String> {
        // Make sure we're authenticated
        let token = {
            let token_guard = self.token.read().await;
            match token_guard.clone() {
                Some(t) => t,
                None => {
                    // Try to authenticate
                    drop(token_guard);
                    self.authenticate().await?;
                    
                    // Get the token again
                    let token_guard = self.token.read().await;
                    match token_guard.clone() {
                        Some(t) => t,
                        None => return Err(AppError::AuthenticationError("Failed to authenticate with HCP".to_string())),
                    }
                }
            }
        };

        let url = format!(
            "{}/secrets/2023-11-28/organizations/{}/projects/{}/apps/{}/secrets/{}:open",
            self.base_url, self.org_id, self.project_id, self.app_name, secret_name
        );

        debug!("Getting secret: {}", secret_name);
        
        let response = self.client
            .get(&url)
            .header(header::AUTHORIZATION, format!("Bearer {}", token))
            .send()
            .await
            .map_err(|e| {
                error!("Failed to get secret from HCP: {}", e);
                AppError::NetworkError(format!("Failed to connect to HCP Secrets: {}", e))
            })?;

        if !response.status().is_success() {
            let status = response.status();
            // For 404 specifically, return NotFoundError
            if status.as_u16() == 404 {
                return Err(AppError::NotFoundError(format!("Secret not found: {}", secret_name)));
            }
            
            let text = response.text().await.unwrap_or_default();
            error!("HCP get secret failed with status {}: {}", status, text);
            return Err(AppError::ServerError(
                anyhow::anyhow!("Failed to retrieve secret from HCP: HTTP {}", status)
            ));
        }

        let secret_response: SecretResponse = response.json().await.map_err(|e| {
            error!("Failed to parse HCP get secret response: {}", e);
            AppError::ServerError(anyhow::anyhow!("Invalid HCP response: {}", e))
        })?;

        debug!("Successfully retrieved secret: {}", secret_name);
        Ok(secret_response.value)
    }

    /// Store a secret in HCP Secrets
    pub async fn store_secret(&self, secret_name: &str, value: &str) -> AppResult<()> {
        // Make sure we're authenticated
        let token = {
            let token_guard = self.token.read().await;
            match token_guard.clone() {
                Some(t) => t,
                None => {
                    // Try to authenticate
                    drop(token_guard);
                    self.authenticate().await?;
                    
                    // Get the token again
                    let token_guard = self.token.read().await;
                    match token_guard.clone() {
                        Some(t) => t,
                        None => return Err(AppError::AuthenticationError("Failed to authenticate with HCP".to_string())),
                    }
                }
            }
        };

        let url = format!(
            "{}/secrets/2023-11-28/organizations/{}/projects/{}/apps/{}/secrets/{}",
            self.base_url, self.org_id, self.project_id, self.app_name, secret_name
        );

        let secret_request = SecretUpsertRequest {
            value: value.to_string(),
        };

        debug!("Storing secret: {}", secret_name);
        
        let response = self.client
            .put(&url)
            .header(header::AUTHORIZATION, format!("Bearer {}", token))
            .header(header::CONTENT_TYPE, "application/json")
            .json(&secret_request)
            .send()
            .await
            .map_err(|e| {
                error!("Failed to store secret in HCP: {}", e);
                AppError::NetworkError(format!("Failed to connect to HCP Secrets: {}", e))
            })?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            error!("HCP store secret failed with status {}: {}", status, text);
            return Err(AppError::ServerError(
                anyhow::anyhow!("Failed to store secret in HCP: HTTP {}", status)
            ));
        }

        debug!("Successfully stored secret: {}", secret_name);
        Ok(())
    }

    /// Helper method to check if the client is authenticated
    pub async fn is_authenticated(&self) -> bool {
        let token_guard = self.token.read().await;
        token_guard.is_some()
    }
}

// #[cfg(test)]
// mod tests {
//     use super::*;
//     use mockito::mock;

//     #[tokio::test]
//     async fn test_authentication_success() {
//         let mock_server = mockito::Server::new();
        
//         // Mock the authentication endpoint
//         let _m = mock_server.mock("POST", "/oauth2/token")
//             .with_status(200)
//             .with_header("content-type", "application/json")
//             .with_body(r#"{"access_token": "test-token", "token_type": "Bearer", "expires_in": 3600}"#)
//             .create();
        
//         let client = SecretsClient::new(
//             &mock_server.url(),
//             "test-org",
//             "test-project",
//             "test-app",
//             "test-client-id",
//             "test-client-secret"
//         );
        
//         let result = client.authenticate().await;
//         assert!(result.is_ok());
        
//         // Verify the token was stored
//         let is_auth = client.is_authenticated().await;
//         assert!(is_auth);
//     }

//     #[tokio::test]
//     async fn test_get_secret_success() {
//         let mock_server = mockito::Server::new();
        
//         // Mock the authentication endpoint
//         let _auth_mock = mock_server.mock("POST", "/oauth2/token")
//             .with_status(200)
//             .with_header("content-type", "application/json")
//             .with_body(r#"{"access_token": "test-token", "token_type": "Bearer", "expires_in": 3600}"#)
//             .create();
        
//         // Mock the get secret endpoint
//         let _secret_mock = mock_server.mock("GET", "/secrets/2023-11-28/organizations/test-org/projects/test-project/apps/test-app/secrets/test-secret:open")
//             .match_header("authorization", "Bearer test-token")
//             .with_status(200)
//             .with_header("content-type", "application/json")
//             .with_body(r#"{"name": "test-secret", "version": 1, "created_at": "2025-01-01T00:00:00Z", "value": "test-value"}"#)
//             .create();
        
//         let client = SecretsClient::new(
//             &mock_server.url(),
//             "test-org",
//             "test-project",
//             "test-app",
//             "test-client-id",
//             "test-client-secret"
//         );
        
//         // Authenticate first
//         let auth_result = client.authenticate().await;
//         assert!(auth_result.is_ok());
        
//         // Get the secret
//         let secret_result = client.get_secret("test-secret").await;
//         assert!(secret_result.is_ok());
//         assert_eq!(secret_result.unwrap(), "test-value");
//     }

//     #[tokio::test]
//     async fn test_store_secret_success() {
//         let mock_server = mockito::Server::new();
        
//         // Mock the authentication endpoint
//         let _auth_mock = mock_server.mock("POST", "/oauth2/token")
//             .with_status(200)
//             .with_header("content-type", "application/json")
//             .with_body(r#"{"access_token": "test-token", "token_type": "Bearer", "expires_in": 3600}"#)
//             .create();
        
//         // Mock the store secret endpoint
//         let _secret_mock = mock_server.mock("PUT", "/secrets/2023-11-28/organizations/test-org/projects/test-project/apps/test-app/secrets/test-secret")
//             .match_header("authorization", "Bearer test-token")
//             .match_header("content-type", "application/json")
//             .match_body(r#"{"value":"test-value"}"#)
//             .with_status(200)
//             .with_header("content-type", "application/json")
//             .with_body(r#"{"name": "test-secret", "version": 1, "created_at": "2025-01-01T00:00:00Z"}"#)
//             .create();
        
//         let client = SecretsClient::new(
//             &mock_server.url(),
//             "test-org",
//             "test-project",
//             "test-app",
//             "test-client-id",
//             "test-client-secret"
//         );
        
//         // Authenticate first
//         let auth_result = client.authenticate().await;
//         assert!(auth_result.is_ok());
        
//         // Store the secret
//         let secret_result = client.store_secret("test-secret", "test-value").await;
//         assert!(secret_result.is_ok());
//     }
// }