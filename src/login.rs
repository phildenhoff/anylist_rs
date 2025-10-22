extern crate reqwest;
extern crate serde;
extern crate serde_derive;

use reqwest::header::{HeaderMap, HeaderValue, CONTENT_TYPE};
use serde_derive::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
struct ApiLoginRequest {
    email: String,
    password: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct ApiTokenResponse {
    access_token: String,
    refresh_token: String,
    user_id: String,
    #[serde(default)]
    is_premium_user: bool,
}

pub struct LoginResult {
    pub is_premium_user: bool,
    pub access_token: String,
    pub refresh_token: String,
    pub user_id: String,
}

pub async fn login(email: &str, password: &str) -> Result<LoginResult, Box<dyn std::error::Error>> {
    let mut headers = HeaderMap::new();
    let client_identifier = format!("anylist-rs-{}", env!("CARGO_PKG_VERSION"));

    headers.insert("X-AnyLeaf-API-Version", HeaderValue::from_static("3"));
    headers.insert(
        "X-AnyLeaf-Client-Identifier",
        HeaderValue::from_str(&client_identifier).unwrap(),
    );
    headers.insert(
        CONTENT_TYPE,
        HeaderValue::from_static("application/json"),
    );

    let request_body = ApiLoginRequest {
        email: email.to_string(),
        password: password.to_string(),
    };

    let client = reqwest::Client::new();
    let res = client
        .post("https://www.anylist.com/auth/token")
        .headers(headers)
        .json(&request_body)
        .send()
        .await?;

    if !res.status().is_success() {
        return Err(format!("Login failed with status: {}", res.status()).into());
    }

    match res.json::<ApiTokenResponse>().await {
        Ok(response) => {
            Ok(LoginResult {
                is_premium_user: response.is_premium_user,
                access_token: response.access_token,
                refresh_token: response.refresh_token,
                user_id: response.user_id,
            })
        }
        Err(e) => Err(Box::new(e)),
    }
}
