use app_error::{AppError, AppResult};
use reqwest::{Client, header};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, error, info};

/// Client for interacting with HashiCorp Vault API
#[derive(Clone)]
pub struct VaultClient {
    client: Client,
    base_url: String,
    token: Arc<RwLock<Option<String>>>,
}

#[derive(Debug, Serialize, Deserialize)]
struct VaultLoginRequest {
    password: String,
}

#[derive(Debug, Deserialize)]
struct VaultLoginResponse {
    auth: VaultAuth,
}

#[derive(Debug, Deserialize)]
struct VaultAuth {
    client_token: String,
}

#[derive(Debug, Serialize)]
struct VaultKVWriteRequest {
    data: HashMap<String, String>,
}

#[derive(Debug, Deserialize)]
struct VaultKVReadResponse {
    data: VaultKVData,
}

#[derive(Debug, Deserialize)]
struct VaultKVData {
    data: HashMap<String, String>,
}

impl VaultClient {
    /// Create a new Vault client
    pub fn new(base_url: &str) -> Self {
        Self {
            client: Client::new(),
            base_url: base_url.to_string(),
            token: Arc::new(RwLock::new(None)),
        }
    }

    /// Login to Vault using username and password
    pub async fn login(&self, username: &str, password: &str) -> AppResult<()> {
        let login_request = VaultLoginRequest {
            password: password.to_string(),
        };

        let url = format!("{}/v1/auth/userpass/login/{}", self.base_url, username);
        let response = self.client
            .post(&url)
            .json(&login_request)
            .send()
            .await
            .map_err(|e| {
                error!("Failed to login to Vault: {}", e);
                AppError::NetworkError(format!("Failed to connect to Vault: {}", e))
            })?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            error!("Vault login failed with status {}: {}", status, text);
            return Err(AppError::AuthenticationError(
                format!("Failed to authenticate with Vault: HTTP {}", status)
            ));
        }

        let login_response: VaultLoginResponse = response.json().await.map_err(|e| {
            error!("Failed to parse Vault login response: {}", e);
            AppError::NetworkError(format!("Invalid Vault response: {}", e))
        })?;

        // Store the token
        let mut token_guard = self.token.write().await;
        *token_guard = Some(login_response.auth.client_token);
        drop(token_guard);

        info!("Successfully authenticated with Vault");
        Ok(())
    }

    /// Store a secret in Vault
    pub async fn store_secret(&self, path: &str, key: &str, value: &str) -> AppResult<()> {
        let token = {
            let token_guard = self.token.read().await;
            match token_guard.clone() {
                Some(t) => t,
                None => return Err(AppError::AuthenticationError("Not authenticated with Vault".to_string())),
            }
        };

        let mut data = HashMap::new();
        data.insert(key.to_string(), value.to_string());

        let write_request = VaultKVWriteRequest { data };
        
        // For KV version 2 engine, path should be in format: data/path
        let url = format!("{}/v1/kv/{}", self.base_url, path);
        
        let response = self.client
            .post(&url)
            .header(header::AUTHORIZATION, format!("Bearer {}", token))
            .json(&write_request)
            .send()
            .await
            .map_err(|e| {
                error!("Failed to store secret in Vault: {}", e);
                AppError::NetworkError(format!("Failed to connect to Vault: {}", e))
            })?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            error!("Vault store secret failed with status {}: {}", status, text);
            return Err(AppError::ServerError(
                anyhow::anyhow!("Failed to store secret in Vault: HTTP {}", status)
            ));
        }

        debug!("Successfully stored secret at path: {}", path);
        Ok(())
    }

    /// Retrieve a secret from Vault
    pub async fn get_secret(&self, path: &str, key: &str) -> AppResult<String> {
        let token = {
            let token_guard = self.token.read().await;
            match token_guard.clone() {
                Some(t) => t,
                None => return Err(AppError::AuthenticationError("Not authenticated with Vault".to_string())),
            }
        };

        // For KV version 2 engine, path should be in format: data/path
        let url = format!("{}/v1/kv/{}", self.base_url, path);
        
        let response = self.client
            .get(&url)
            .header(header::AUTHORIZATION, format!("Bearer {}", token))
            .send()
            .await
            .map_err(|e| {
                error!("Failed to retrieve secret from Vault: {}", e);
                AppError::NetworkError(format!("Failed to connect to Vault: {}", e))
            })?;

        if !response.status().is_success() {
            let status = response.status();
            // For 404 specifically, return None
            if status.as_u16() == 404 {
                return Err(AppError::NotFoundError(format!("Secret not found at path: {}", path)));
            }
            
            let text = response.text().await.unwrap_or_default();
            error!("Vault get secret failed with status {}: {}", status, text);
            return Err(AppError::ServerError(
                anyhow::anyhow!("Failed to retrieve secret from Vault: HTTP {}", status)
            ));
        }

        let read_response: VaultKVReadResponse = response.json().await.map_err(|e| {
            error!("Failed to parse Vault get secret response: {}", e);
            AppError::ServerError(anyhow::anyhow!("Invalid Vault response: {}", e))
        })?;

        match read_response.data.data.get(key) {
            Some(value) => {
                debug!("Successfully retrieved secret at path: {}", path);
                Ok(value.to_string())
            },
            None => Err(AppError::NotFoundError(format!("Key '{}' not found in secret at path: {}", key, path))),
        }
    }

    /// Helper method to check if the client is authenticated
    pub async fn is_authenticated(&self) -> bool {
        let token_guard = self.token.read().await;
        token_guard.is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wiremock::{MockServer, Mock, ResponseTemplate};
    use wiremock::matchers::{method, path, header, body_json_string};
    use serde_json::json;

    #[tokio::test]
    async fn test_login() -> Result<(), anyhow::Error> {
        // Start a mock server
        let mock_server = MockServer::start().await;
        
        // Set up mock for login request
        Mock::given(method("POST"))
            .and(path("/v1/auth/userpass/login/testuser"))
            .and(body_json_string(r#"{"password":"password"}"#))
            .respond_with(ResponseTemplate::new(200)
                .set_body_json(json!({
                    "auth": {
                        "client_token": "test-token"
                    }
                }))
            )
            .mount(&mock_server)
            .await;

        // Create client and test login
        let client = VaultClient::new(&mock_server.uri());
        client.login("testuser", "password").await?;
        
        // Verify client is authenticated
        assert!(client.is_authenticated().await);
        
        Ok(())
    }

    #[tokio::test]
    async fn test_login_failure() -> Result<(), anyhow::Error> {
        // Start a mock server
        let mock_server = MockServer::start().await;
        
        // Set up mock for failed login
        Mock::given(method("POST"))
            .and(path("/v1/auth/userpass/login/testuser"))
            .respond_with(ResponseTemplate::new(403)
                .set_body_string("permission denied")
            )
            .mount(&mock_server)
            .await;

        // Create client and test login failure
        let client = VaultClient::new(&mock_server.uri());
        let result = client.login("testuser", "wrong-password").await;
        
        // Verify authentication failed
        assert!(result.is_err());
        assert!(!client.is_authenticated().await);
        
        Ok(())
    }

    #[tokio::test]
    async fn test_store_secret() -> Result<(), anyhow::Error> {
        // Start a mock server
        let mock_server = MockServer::start().await;
        
        // Set up mock for login
        Mock::given(method("POST"))
            .and(path("/v1/auth/userpass/login/testuser"))
            .respond_with(ResponseTemplate::new(200)
                .set_body_json(json!({
                    "auth": {
                        "client_token": "test-token"
                    }
                }))
            )
            .mount(&mock_server)
            .await;
        
        // Set up mock for store secret
        Mock::given(method("POST"))
            .and(path("/v1/kv/test/path"))
            .and(header("authorization", "Bearer test-token"))
            .and(body_json_string(r#"{"data":{"test-key":"test-value"}}"#))
            .respond_with(ResponseTemplate::new(200)
                .set_body_json(json!({}))
            )
            .mount(&mock_server)
            .await;

        // Create client, login, and test storing a secret
        let client = VaultClient::new(&mock_server.uri());
        client.login("testuser", "password").await?;
        
        let result = client.store_secret("test/path", "test-key", "test-value").await;
        assert!(result.is_ok());
        
        Ok(())
    }

    #[tokio::test]
    async fn test_get_secret() -> Result<(), anyhow::Error> {
        // Start a mock server
        let mock_server = MockServer::start().await;
        
        // Set up mock for login
        Mock::given(method("POST"))
            .and(path("/v1/auth/userpass/login/testuser"))
            .respond_with(ResponseTemplate::new(200)
                .set_body_json(json!({
                    "auth": {
                        "client_token": "test-token"
                    }
                }))
            )
            .mount(&mock_server)
            .await;
        
        // Set up mock for get secret
        Mock::given(method("GET"))
            .and(path("/v1/kv/test/path"))
            .and(header("authorization", "Bearer test-token"))
            .respond_with(ResponseTemplate::new(200)
                .set_body_json(json!({
                    "data": {
                        "data": {
                            "test-key": "test-value"
                        }
                    }
                }))
            )
            .mount(&mock_server)
            .await;

        // Create client, login, and test getting a secret
        let client = VaultClient::new(&mock_server.uri());
        client.login("testuser", "password").await?;
        
        let value = client.get_secret("test/path", "test-key").await?;
        assert_eq!(value, "test-value");
        
        Ok(())
    }

    #[tokio::test]
    async fn test_get_nonexistent_secret() -> Result<(), anyhow::Error> {
        // Start a mock server
        let mock_server = MockServer::start().await;
        
        // Set up mock for login
        Mock::given(method("POST"))
            .and(path("/v1/auth/userpass/login/testuser"))
            .respond_with(ResponseTemplate::new(200)
                .set_body_json(json!({
                    "auth": {
                        "client_token": "test-token"
                    }
                }))
            )
            .mount(&mock_server)
            .await;
        
        // Set up mock for get nonexistent secret (404)
        Mock::given(method("GET"))
            .and(path("/v1/kv/nonexistent/path"))
            .and(header("authorization", "Bearer test-token"))
            .respond_with(ResponseTemplate::new(404)
                .set_body_json(json!({
                    "errors": ["secret not found"]
                }))
            )
            .mount(&mock_server)
            .await;

        // Create client, login, and test getting a nonexistent secret
        let client = VaultClient::new(&mock_server.uri());
        client.login("testuser", "password").await?;
        
        let result = client.get_secret("nonexistent/path", "test-key").await;
        assert!(result.is_err());
        
        // Check that it's specifically a NotFoundError
        match result {
            Err(AppError::NotFoundError(_)) => assert!(true),
            _ => assert!(false, "Expected NotFoundError but got a different result"),
        }
        
        Ok(())
    }

    #[tokio::test]
    async fn test_not_authenticated() -> Result<(), anyhow::Error> {
        // Start a mock server
        let mock_server = MockServer::start().await;
        
        // Create client without login
        let client = VaultClient::new(&mock_server.uri());
        
        // Try to store a secret without authentication
        let store_result = client.store_secret("test/path", "test-key", "test-value").await;
        assert!(store_result.is_err());
        
        // Try to get a secret without authentication
        let get_result = client.get_secret("test/path", "test-key").await;
        assert!(get_result.is_err());
        
        // Check that both errors are AuthenticationError
        match store_result {
            Err(AppError::AuthenticationError(_)) => assert!(true),
            _ => assert!(false, "Expected AuthenticationError but got a different result"),
        }
        
        match get_result {
            Err(AppError::AuthenticationError(_)) => assert!(true),
            _ => assert!(false, "Expected AuthenticationError but got a different result"),
        }
        
        Ok(())
    }
}