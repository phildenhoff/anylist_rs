use crate::error::{AnyListError, Result};
use crate::login::{login, LoginResult};
use reqwest::header::{HeaderMap, HeaderValue};

/// Main client for interacting with the AnyList API.
///
/// To re-use an existing login session, save [AnyListClient::signed_user_id],
/// [AnyListClient::user_id], and [AnyListClient::is_premium_user] then, later,
/// call [AnyListClient::from_credentials]
pub struct AnyListClient {
    /// Used as a credential to authorize requests, obtained during a call to [AnyListClient::new]
    pub signed_user_id: String,
    pub user_id: String,
    pub is_premium_user: bool,
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

    /// Create an AnyList client from existing login credentials
    ///
    /// # Arguments
    ///
    /// * `signed_user_id` - The signed user ID from a previous login
    /// * `user_id` - The user ID from a previous login
    /// * `is_premium_user` - Whether the user has premium subscription
    pub fn from_credentials(
        signed_user_id: String,
        user_id: String,
        is_premium_user: bool,
    ) -> Self {
        Self {
            signed_user_id,
            user_id,
            is_premium_user,
            client: reqwest::Client::new(),
        }
    }

    /// Create an AnyList client from a login result
    fn from_login_result(login_result: LoginResult) -> Self {
        Self {
            signed_user_id: login_result.credential,
            user_id: login_result.user_id,
            is_premium_user: login_result.is_premium_user,
            client: reqwest::Client::new(),
        }
    }

    /// Get default headers for API requests
    fn get_headers(&self) -> HeaderMap {
        let mut headers = HeaderMap::new();
        headers.insert(
            "X-AnyLeaf-API-Version",
            HeaderValue::from_static("3"),
        );
        headers.insert(
            "X-AnyLeaf-Signed-User-ID",
            HeaderValue::from_str(&self.signed_user_id).unwrap(),
        );
        headers.insert(
            "Content-Type",
            HeaderValue::from_static("application/x-protobuf"),
        );
        headers
    }

    /// Make a POST request to the AnyList API
    pub(crate) async fn post(&self, endpoint: &str, body: Vec<u8>) -> Result<Vec<u8>> {
        let url = format!("https://www.anylist.com/data/{}", endpoint);
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
