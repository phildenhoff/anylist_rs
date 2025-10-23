use reqwest::header::HeaderMap;
use serde_derive::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
struct TokenResponse {
    access_token: String,
    refresh_token: String,
    user_id: String,
    #[serde(default)]
    is_premium_user: bool,
}

pub(crate) struct LoginResult {
    pub access_token: String,
    pub refresh_token: String,
    pub user_id: String,
    pub is_premium_user: bool,
}

pub(crate) async fn login(
    email: &str,
    password: &str,
    client_identifier: &str,
) -> Result<LoginResult, Box<dyn std::error::Error>> {
    let mut headers = HeaderMap::new();
    headers.insert("X-AnyLeaf-API-Version", "3".parse().unwrap());
    headers.insert(
        "X-AnyLeaf-Client-Identifier",
        client_identifier.parse().unwrap(),
    );

    let form = reqwest::multipart::Form::new()
        .text("email", email.to_string())
        .text("password", password.to_string());

    let client = reqwest::Client::new();
    let res = client
        .post("https://www.anylist.com/auth/token")
        .headers(headers)
        .multipart(form)
        .send()
        .await?;

    if !res.status().is_success() {
        let status = res.status();
        let headers = res.headers().clone();
        let body = res.text().await?;

        eprintln!("Login failed:");
        eprintln!("  Status: {}", status);
        eprintln!("  Headers: {:?}", headers);
        eprintln!("  Body: {}", body);

        return Err(format!("Login failed with status: {}, body: {}", status, body).into());
    }

    match res.json::<TokenResponse>().await {
        Ok(response) => Ok(LoginResult {
            access_token: response.access_token,
            refresh_token: response.refresh_token,
            user_id: response.user_id,
            is_premium_user: response.is_premium_user,
        }),
        Err(e) => Err(Box::new(e)),
    }
}
