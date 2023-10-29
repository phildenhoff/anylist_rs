mod login;

extern crate clap;
use clap::Command;
use login::login_subcommand;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let matches = Command::new("AnyList Client")
        .subcommand(login_subcommand())
        .get_matches();

    if let Some(matches) = matches.subcommand_matches("login") {
        let email = matches.value_of("email").unwrap();
        let password = matches.value_of("password").unwrap();
        login::login(email, password).await?;
    }

    Ok(())
}
