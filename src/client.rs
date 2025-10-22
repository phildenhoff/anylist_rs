use crate::error::{AnyListError, Result};
use crate::login::{login, LoginResult};
use crate::utils::generate_id;
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION};
use serde_derive::{Deserialize, Serialize};

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
    pub access_token: String,
    /// Refresh token for obtaining new access tokens
    pub refresh_token: String,
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
            access_token,
            refresh_token,
            user_id,
            is_premium_user,
            client_identifier: generate_id(),
            client: reqwest::Client::new(),
        }
    }

    /// Create an AnyList client from a login result
    fn from_login_result(login_result: LoginResult) -> Self {
        Self {
            access_token: login_result.access_token,
            refresh_token: login_result.refresh_token,
            user_id: login_result.user_id,
            is_premium_user: login_result.is_premium_user,
            client_identifier: generate_id(),
            client: reqwest::Client::new(),
        }
    }

    /// Refresh the access token using the refresh token
    ///
    /// Returns a new client with updated tokens. The old client's tokens will be invalid.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use anylist_rs::AnyListClient;
    /// # #[tokio::main]
    /// # async fn main() {
    /// let client = AnyListClient::new("user@example.com", "password")
    ///     .await
    ///     .expect("Failed to authenticate");
    ///
    /// // Later, when token expires...
    /// let client = client.refresh().await.expect("Failed to refresh token");
    /// # }
    /// ```
    pub async fn refresh(self) -> Result<Self> {
        let request_body = TokenRefreshRequest {
            refresh_token: self.refresh_token.clone(),
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

        Ok(Self {
            access_token: token_response.access_token,
            refresh_token: token_response.refresh_token,
            user_id: self.user_id,
            is_premium_user: self.is_premium_user,
            client_identifier: self.client_identifier,
            client: self.client,
        })
    }

    /// Get default headers for API requests
    fn get_headers(&self) -> HeaderMap {
        let mut headers = HeaderMap::new();

        let bearer_value = format!("Bearer {}", self.access_token);

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

    /// Make a POST request to the AnyList API
    ///
    /// If you receive a 401 Unauthorized error, call `client.refresh().await?` to get
    /// a new client with refreshed tokens.
    pub(crate) async fn post(&self, endpoint: &str, body: Vec<u8>) -> Result<Vec<u8>> {
        let url = format!("https://www.anylist.com/{}", endpoint);

        let response = self
            .client
            .post(&url)
            .headers(self.get_headers())
            .body(body)
            .send()
            .await?;

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
