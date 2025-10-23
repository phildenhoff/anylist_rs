use crate::error::{AnyListError, Result};
use crate::login::login;
use crate::utils::generate_id;
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION};
use serde_derive::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};

// ============================================================================
// Public types for persistence and events
// ============================================================================

/// Tokens that can be saved and restored for persistent sessions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SavedTokens {
    pub access_token: String,
    pub refresh_token: String,
    pub user_id: String,
    pub is_premium_user: bool,
}

/// Authentication events that can be monitored
#[derive(Debug, Clone)]
pub enum AuthEvent {
    /// Tokens were successfully refreshed
    TokensRefreshed,
    /// Token refresh failed
    RefreshFailed(String),
}

// ============================================================================
// Internal auth types
// ============================================================================

#[derive(Clone)]
struct AuthState {
    access_token: String,
    refresh_token: String,
    user_id: String,
    is_premium_user: bool,
    auto_refresh_enabled: bool,
}

/// Main client for interacting with the AnyList API.
///
/// Automatically manages authentication tokens and handles token refresh.
/// Use `export_tokens()` to persist sessions and `from_tokens()` to restore them.
///
/// # Example
///
/// ```no_run
/// use anylist_rs::AnyListClient;
///
/// #[tokio::main]
/// async fn main() {
///     // Login with email and password
///     let client = AnyListClient::login("user@example.com", "password")
///         .await
///         .expect("Failed to login");
///
///     // Use the client - tokens are managed automatically
///     let lists = client.get_lists().await.expect("Failed to get lists");
///
///     // Export tokens for later use
///     let tokens = client.export_tokens().await.expect("Failed to export");
///     // Save tokens to disk/keychain...
///
///     // Restore from saved tokens
///     let client = AnyListClient::from_tokens(tokens).expect("Failed to restore");
/// }
/// ```
pub struct AnyListClient {
    /// Internal authentication state (managed automatically)
    auth: Arc<Mutex<AuthState>>,
    /// Optional callback for auth events
    auth_event_callback: Option<Arc<dyn Fn(AuthEvent) + Send + Sync>>,
    /// Unique client identifier (UUID)
    client_identifier: String,
    /// HTTP client for making requests
    client: reqwest::Client,
}

impl AnyListClient {
    /// Create a new AnyList client by logging in with email and password.
    ///
    /// This automatically acquires access and refresh tokens.
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
    ///     let client = AnyListClient::login("user@example.com", "password")
    ///         .await
    ///         .expect("Failed to authenticate");
    ///
    ///     let lists = client.get_lists().await.expect("Failed to get lists");
    /// }
    /// ```
    pub async fn login(email: &str, password: &str) -> Result<Self> {
        let client_identifier = generate_id();

        let login_result = login(email, password, &client_identifier)
            .await
            .map_err(|e| AnyListError::AuthenticationFailed(e.to_string()))?;

        let auth = Arc::new(Mutex::new(AuthState {
            access_token: login_result.access_token,
            refresh_token: login_result.refresh_token,
            user_id: login_result.user_id,
            is_premium_user: login_result.is_premium_user,
            auto_refresh_enabled: true,
        }));

        Ok(Self {
            auth,
            auth_event_callback: None,
            client_identifier,
            client: reqwest::Client::new(),
        })
    }

    /// Create an AnyList client from previously saved tokens.
    ///
    /// Use this to restore a session without logging in again.
    ///
    /// # Arguments
    ///
    /// * `tokens` - Previously saved tokens from `export_tokens()`
    ///
    /// # Example
    ///
    /// ```no_run
    /// use anylist_rs::{AnyListClient, SavedTokens};
    ///
    /// # fn load_from_keychain() -> Result<SavedTokens, Box<dyn std::error::Error>> { todo!() }
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// // Load tokens from storage
    /// let tokens: SavedTokens = load_from_keychain()?;
    ///
    /// let client = AnyListClient::from_tokens(tokens)?;
    /// let lists = client.get_lists().await?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn from_tokens(tokens: SavedTokens) -> Result<Self> {
        let auth = Arc::new(Mutex::new(AuthState {
            access_token: tokens.access_token,
            refresh_token: tokens.refresh_token,
            user_id: tokens.user_id,
            is_premium_user: tokens.is_premium_user,
            auto_refresh_enabled: true,
        }));

        Ok(Self {
            auth,
            auth_event_callback: None,
            client_identifier: generate_id(),
            client: reqwest::Client::new(),
        })
    }

    /// Export tokens for persistent storage.
    ///
    /// Save these tokens to restore the session later without re-authenticating.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use anylist_rs::AnyListClient;
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = AnyListClient::login("user@example.com", "password").await?;
    ///
    /// // Export and save tokens
    /// let tokens = client.export_tokens().await?;
    /// // save_to_keychain(&tokens)?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn export_tokens(&self) -> Result<SavedTokens> {
        let auth = self.auth.lock().unwrap();

        Ok(SavedTokens {
            access_token: auth.access_token.clone(),
            refresh_token: auth.refresh_token.clone(),
            user_id: auth.user_id.clone(),
            is_premium_user: auth.is_premium_user,
        })
    }

    /// Register a callback for authentication events.
    ///
    /// The callback will be invoked when tokens are acquired, refreshed, or refresh fails.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use anylist_rs::{AnyListClient, AuthEvent};
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = AnyListClient::login("user@example.com", "password")
    ///     .await?
    ///     .on_auth_event(|event| {
    ///         match event {
    ///             AuthEvent::TokensRefreshed => println!("Tokens refreshed!"),
    ///             AuthEvent::RefreshFailed(err) => eprintln!("Refresh failed: {}", err),
    ///             _ => {}
    ///         }
    ///     });
    /// # Ok(())
    /// # }
    /// ```
    pub fn on_auth_event<F>(mut self, callback: F) -> Self
    where
        F: Fn(AuthEvent) + Send + Sync + 'static,
    {
        self.auth_event_callback = Some(Arc::new(callback));
        self
    }

    /// Disable automatic token refresh.
    ///
    /// When disabled, the client will return errors on 401 instead of automatically
    /// refreshing tokens. Useful if you want full control over when refreshes happen.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use anylist_rs::AnyListClient;
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = AnyListClient::login("user@example.com", "password")
    ///     .await?
    ///     .disable_auto_refresh();
    /// # Ok(())
    /// # }
    /// ```
    pub fn disable_auto_refresh(self) -> Self {
        let mut auth = self.auth.lock().unwrap();
        auth.auto_refresh_enabled = false;
        drop(auth);
        self
    }

    /// Get the user ID for this client.
    pub fn user_id(&self) -> String {
        let auth = self.auth.lock().unwrap();
        auth.user_id.clone()
    }

    /// Check if the user has premium subscription.
    pub fn is_premium_user(&self) -> bool {
        let auth = self.auth.lock().unwrap();
        auth.is_premium_user
    }

    // ========================================================================
    // Internal authentication methods
    // ========================================================================

    /// Refresh the access token using the refresh token.
    ///
    /// This calls /auth/token/refresh endpoint with multipart form data
    async fn refresh_tokens(&self) -> Result<()> {
        let refresh_token = {
            let auth = self.auth.lock().unwrap();
            auth.refresh_token.clone()
        };

        let mut headers = HeaderMap::new();
        headers.insert("X-AnyLeaf-API-Version", HeaderValue::from_static("3"));
        headers.insert(
            "X-AnyLeaf-Client-Identifier",
            HeaderValue::from_str(&self.client_identifier).unwrap(),
        );

        let form = reqwest::multipart::Form::new().text("refresh_token", refresh_token);

        let response = self
            .client
            .post("https://www.anylist.com/auth/token/refresh")
            .headers(headers)
            .multipart(form)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await?;
            let error_msg = format!(
                "Token refresh failed with status: {}, body: {}",
                status, body
            );

            // Notify callback of failure
            if let Some(callback) = &self.auth_event_callback {
                callback(AuthEvent::RefreshFailed(error_msg.clone()));
            }

            return Err(AnyListError::AuthenticationFailed(error_msg));
        }

        #[derive(Deserialize)]
        struct RefreshResponse {
            access_token: String,
            refresh_token: String,
        }

        let token_response: RefreshResponse = response.json().await?;

        // Update auth state
        {
            let mut auth = self.auth.lock().unwrap();
            auth.access_token = token_response.access_token;
            auth.refresh_token = token_response.refresh_token;
        }

        // Notify callback
        if let Some(callback) = &self.auth_event_callback {
            callback(AuthEvent::TokensRefreshed);
        }

        Ok(())
    }

    /// Get default headers for API requests
    fn get_headers(&self) -> HeaderMap {
        let mut headers = HeaderMap::new();

        let auth = self.auth.lock().unwrap();
        let bearer_value = format!("Bearer {}", auth.access_token);
        headers.insert(AUTHORIZATION, HeaderValue::from_str(&bearer_value).unwrap());
        drop(auth);

        headers.insert("X-AnyLeaf-API-Version", HeaderValue::from_static("3"));
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

    /// Make a POST request to the AnyList API.
    ///
    /// Automatically handles token refresh on 401 errors if auto_refresh is enabled.
    pub(crate) async fn post(&self, endpoint: &str, body: Vec<u8>) -> Result<Vec<u8>> {
        let url = format!("https://www.anylist.com/{}", endpoint);

        let response = self
            .client
            .post(&url)
            .headers(self.get_headers())
            .body(body.clone())
            .send()
            .await?;

        // Handle 401 with automatic token refresh
        if response.status() == 401 {
            let auto_refresh = {
                let auth = self.auth.lock().unwrap();
                auth.auto_refresh_enabled
            };

            if auto_refresh {
                // Try to refresh tokens
                self.refresh_tokens().await?;

                // Retry the request with new token
                let retry_response = self
                    .client
                    .post(&url)
                    .headers(self.get_headers())
                    .body(body)
                    .send()
                    .await?;

                if !retry_response.status().is_success() {
                    return Err(AnyListError::NetworkError(format!(
                        "Request failed after token refresh with status: {}",
                        retry_response.status()
                    )));
                }

                let bytes = retry_response.bytes().await?;
                return Ok(bytes.to_vec());
            } else {
                return Err(AnyListError::AuthenticationFailed(
                    "Unauthorized (auto-refresh disabled)".to_string(),
                ));
            }
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
