extern crate clap;
extern crate reqwest;
extern crate serde;
extern crate serde_derive;

use clap::{Arg, Command, SubCommand};
use reqwest::header::{HeaderMap, HeaderValue, CONTENT_TYPE};
use serde_derive::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
struct LoginResponse {
    is_premium_user: bool,
    signed_user_id: String,
    user_id: String,
}

pub async fn login(email: &str, password: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut headers = HeaderMap::new();
    headers.insert("X-AnyLeaf-API-Version", HeaderValue::from_static("3"));
    headers.insert(
        "X-AnyLeaf-Client-Identifier",
        HeaderValue::from_static("asejfklasdfj"),
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

    match res.json::<LoginResponse>().await {
        Ok(response) => {
            println!("Is premium user?: {:?}", response.is_premium_user);
            println!("Signed user ID: {:?}", response.signed_user_id);
        }
        Err(e) => println!("Error: {:?}", e),
    }

    Ok(())
}

pub fn login_subcommand() -> Command<'static> {
    return SubCommand::with_name("login")
        .arg(
            Arg::with_name("email")
                .short('e')
                .long("email")
                .value_name("EMAIL")
                .help("Sets the email to use")
                .takes_value(true)
                .required(true),
        )
        .arg(
            Arg::with_name("password")
                .short('p')
                .long("password")
                .value_name("PASSWORD")
                .help("Sets the password to use")
                .takes_value(true)
                .required(true),
        );
}
