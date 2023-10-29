mod login;

extern crate clap;
use clap::{App, Arg, SubCommand};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let matches = App::new("AnyList Client")
        .subcommand(
            SubCommand::with_name("login")
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
                ),
        )
        .get_matches();

    if let Some(matches) = matches.subcommand_matches("login") {
        let email = matches.value_of("email").unwrap();
        let password = matches.value_of("password").unwrap();
        login::login(email, password).await?;
    }

    Ok(())
}
