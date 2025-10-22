use crate::error::{AnyListError, Result};
use crate::login::{login, LoginResult};
use crate::utils::generate_id;
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION};
use serde_derive::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};

#[derive(Serialize, Deserialize, Debug)]
struct TokenRefreshRequest {
    refresh_token: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct TokenRefreshResponse {
    access_token: String,
    refresh_token: String,
}

/// Main client for interacting with the AnyList API.
///
/// To re-use an existing login session, save [AnyListClient::access_token],
/// [AnyListClient::refresh_token], [AnyListClient::user_id], and
/// [AnyListClient::is_premium_user] then, later, call [AnyListClient::from_tokens]
pub struct AnyListClient {
    /// Access token for authentication (Bearer token)
    access_token: Arc<Mutex<String>>,
    /// Refresh token for obtaining new access tokens
    refresh_token: Arc<Mutex<String>>,
    /// User ID
    pub user_id: String,
    /// Whether the user has premium subscription
    pub is_premium_user: bool,
    /// Unique client identifier (UUID)
    client_identifier: String,
    /// HTTP client for making requests
    client: reqwest::Client,
}

impl AnyListClient {
    /// Create a new AnyList client by logging in with email and password
    ///
    /// # Arguments
    ///
    /// * `email` - User's email address
    /// * `password` - User's password
    ///
    /// # Example
    ///
    /// ```no_run
    /// use anylist_rs::AnyListClient;
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     let client = AnyListClient::new("user@example.com", "password")
    ///         .await
    ///         .expect("Failed to authenticate");
    /// }
    /// ```
    pub async fn new(email: &str, password: &str) -> Result<Self> {
        let login_result = login(email, password)
            .await
            .map_err(|e| AnyListError::AuthenticationFailed(e.to_string()))?;

        Ok(Self::from_login_result(login_result))
    }

    /// Create an AnyList client from existing tokens
    ///
    /// # Arguments
    ///
    /// * `access_token` - The access token from a previous login
    /// * `refresh_token` - The refresh token from a previous login
    /// * `user_id` - The user ID from a previous login
    /// * `is_premium_user` - Whether the user has premium subscription
    ///
    /// # Example
    ///
    /// ```no_run
    /// use anylist_rs::AnyListClient;
    ///
    /// let client = AnyListClient::from_tokens(
    ///     "access_token_here".to_string(),
    ///     "refresh_token_here".to_string(),
    ///     "user_id_here".to_string(),
    ///     false,
    /// );
    /// ```
    pub fn from_tokens(
        access_token: String,
        refresh_token: String,
        user_id: String,
        is_premium_user: bool,
    ) -> Self {
        Self {
            access_token: Arc::new(Mutex::new(access_token)),
            refresh_token: Arc::new(Mutex::new(refresh_token)),
            user_id,
            is_premium_user,
            client_identifier: generate_id(),
            client: reqwest::Client::new(),
        }
    }

    /// Create an AnyList client from a login result
    fn from_login_result(login_result: LoginResult) -> Self {
        Self {
            access_token: Arc::new(Mutex::new(login_result.access_token)),
            refresh_token: Arc::new(Mutex::new(login_result.refresh_token)),
            user_id: login_result.user_id,
            is_premium_user: login_result.is_premium_user,
            client_identifier: generate_id(),
            client: reqwest::Client::new(),
        }
    }

    /// Get the current access token
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use anylist_rs::AnyListClient;
    /// # #[tokio::main]
    /// # async fn main() {
    /// # let client = AnyListClient::new("user@example.com", "password")
    /// #     .await
    /// #     .expect("Failed to authenticate");
    /// let access_token = client.get_access_token();
    /// println!("Access token: {}", access_token);
    /// # }
    /// ```
    pub fn get_access_token(&self) -> String {
        self.access_token.lock().unwrap().clone()
    }

    /// Get the current refresh token
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use anylist_rs::AnyListClient;
    /// # #[tokio::main]
    /// # async fn main() {
    /// # let client = AnyListClient::new("user@example.com", "password")
    /// #     .await
    /// #     .expect("Failed to authenticate");
    /// let refresh_token = client.get_refresh_token();
    /// println!("Refresh token: {}", refresh_token);
    /// # }
    /// ```
    pub fn get_refresh_token(&self) -> String {
        self.refresh_token.lock().unwrap().clone()
    }

    /// Refresh the access token using the refresh token
    async fn refresh_token(&self) -> Result<()> {
        let refresh_token = self.refresh_token.lock().unwrap().clone();

        let request_body = TokenRefreshRequest {
            refresh_token,
        };

        let mut headers = HeaderMap::new();
        headers.insert("X-AnyLeaf-API-Version", HeaderValue::from_static("3"));
        headers.insert(
            "X-AnyLeaf-Client-Identifier",
            HeaderValue::from_str(&self.client_identifier).unwrap(),
        );
        headers.insert(
            "Content-Type",
            HeaderValue::from_static("application/json"),
        );

        let response = self
            .client
            .post("https://www.anylist.com/auth/token/refresh")
            .headers(headers)
            .json(&request_body)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(AnyListError::AuthenticationFailed(format!(
                "Token refresh failed with status: {}",
                response.status()
            )));
        }

        let token_response: TokenRefreshResponse = response.json().await?;

        *self.access_token.lock().unwrap() = token_response.access_token;
        *self.refresh_token.lock().unwrap() = token_response.refresh_token;

        Ok(())
    }

    /// Get default headers for API requests
    fn get_headers(&self) -> HeaderMap {
        let mut headers = HeaderMap::new();

        let access_token = self.access_token.lock().unwrap().clone();
        let bearer_value = format!("Bearer {}", access_token);

        headers.insert(
            AUTHORIZATION,
            HeaderValue::from_str(&bearer_value).unwrap(),
        );
        headers.insert(
            "X-AnyLeaf-API-Version",
            HeaderValue::from_static("3"),
        );
        headers.insert(
            "X-AnyLeaf-Client-Identifier",
            HeaderValue::from_str(&self.client_identifier).unwrap(),
        );
        headers.insert(
            "Content-Type",
            HeaderValue::from_static("application/x-protobuf"),
        );
        headers
    }

    /// Make a POST request to the AnyList API with automatic token refresh on 401
    pub(crate) async fn post(&self, endpoint: &str, body: Vec<u8>) -> Result<Vec<u8>> {
        let url = format!("https://www.anylist.com/{}", endpoint);

        // Try the request with current token
        let response = self
            .client
            .post(&url)
            .headers(self.get_headers())
            .body(body.clone())
            .send()
            .await?;

        // If 401 Unauthorized, try to refresh token and retry once
        if response.status() == reqwest::StatusCode::UNAUTHORIZED {
            self.refresh_token().await?;

            // Retry with new token
            let response = self
                .client
                .post(&url)
                .headers(self.get_headers())
                .body(body)
                .send()
                .await?;

            if !response.status().is_success() {
                return Err(AnyListError::NetworkError(format!(
                    "Request failed with status: {} after token refresh",
                    response.status()
                )));
            }

            let bytes = response.bytes().await?;
            return Ok(bytes.to_vec());
        }

        if !response.status().is_success() {
            return Err(AnyListError::NetworkError(format!(
                "Request failed with status: {}",
                response.status()
            )));
        }

        let bytes = response.bytes().await?;
        Ok(bytes.to_vec())
    }
}
