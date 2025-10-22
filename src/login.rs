extern crate reqwest;
extern crate serde;
extern crate serde_derive;

use reqwest::header::{HeaderMap, HeaderValue, CONTENT_TYPE};
use serde_derive::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
struct ApiLoginResponse {
    is_premium_user: bool,
    signed_user_id: String,
    user_id: String,
}

pub struct LoginResult {
    pub is_premium_user: bool,
    pub credential: String,
    pub user_id: String,
}

pub async fn login(email: &str, password: &str) -> Result<LoginResult, Box<dyn std::error::Error>> {
    let mut headers = HeaderMap::new();
    let client_identifer = format!("anylist-rs-{}", env!("CARGO_PKG_VERSION"));

    headers.insert("X-AnyLeaf-API-Version", HeaderValue::from_static("3"));
    headers.insert(
        "X-AnyLeaf-Client-Identifier",
        HeaderValue::from_str(&client_identifer).unwrap(),
    );
    headers.insert(
        CONTENT_TYPE,
        HeaderValue::from_static("application/x-www-form-urlencoded"),
    );

    let client = reqwest::Client::new();
    let res = client
        .post("https://www.anylist.com/data/validate-login")
        .headers(headers)
        .form(&[("email", email), ("password", password)])
        .send()
        .await?;

    match res.json::<ApiLoginResponse>().await {
        Ok(response) => {
            Ok(LoginResult {
                is_premium_user: response.is_premium_user,
                credential: response.signed_user_id,
                user_id: response.user_id,
            })
        }
        Err(e) => Err(Box::new(e)),
    }
}
