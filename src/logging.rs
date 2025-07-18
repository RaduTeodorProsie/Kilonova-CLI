use super::credential_manager::CredentialManager;
use super::{credential_manager, waiter};
use colored::Colorize;
use indicatif::{ProgressBar, ProgressStyle};
use reqwest::blocking::Client;
use serde_json::Value;
use std::io;
use std::io::Write;
use std::time::Duration;
use waiter::Waiter;

fn read_username_and_password() -> (String, String) {
    use rpassword::read_password;

    print!("Username: ");
    io::stdout().flush().unwrap();

    let mut username = String::new();
    io::stdin()
        .read_line(&mut username)
        .expect("Failed to read username");
    let username: String = username.trim().parse().expect("Only valid characters");

    print!("Password: ");
    io::stdout().flush().unwrap();
    let password = read_password().expect("Failed to read password");

    (username, password)
}

fn internal_login(username : String, password : String) -> Result<String, reqwest::Error> {
    let client = Client::new();
    let response_text = client
        .post("https://kilonova.ro/api/auth/login")
        .query(&[("username", username), ("password", password)])
        .timeout(Duration::from_secs(10))
        .send()?
        .text()?;
    Ok(response_text)
}

fn login_and_print(username: String, password: String) {
    let spinner = Waiter::start();

    let response = internal_login(username, password);
    let response = match response {
        Ok(response) => response,
        Err(e) => {
            println!("Ran into error: {}", e.to_string());
            return ()
        }
    };


    let response: Value = serde_json::from_str(response.as_ref()).unwrap();
    let status = response["status"].as_str().unwrap();
    let data = response["data"].as_str().unwrap();

    spinner.stop();

    match status {
        "success" => {
            println!("{}", "Successfully logged in ✅".green());
            use super::credential_manager;
            CredentialManager::global()
                .set::<credential_manager::Token>(data)
                .expect("Could not cache the token");
        }
        _ => {
            println!("{}", "Wrong credentials".red());
        }
    }
}

pub fn login() {
    let (username, password) = read_username_and_password();
    login_and_print(username, password);
}

pub fn logout() {
    let spinner = Waiter::start();
    let token = CredentialManager::global()
        .get::<credential_manager::Token>()
        .expect("Could not get token. Maybe you are not logged in?");
    Client::new()
        .post("https://kilonova.ro/api/auth/logout")
        .header("Authorization", token)
        .timeout(Duration::from_secs(10))
        .send()
        .expect("Could not send the request to logout or the site timed out");
    CredentialManager::global().delete::<credential_manager::Token>().expect("Could not delete the token");

    spinner.stop();
    println!("{}", "Successfully logged out ✅".green());
}

mod tests {
    #[test]
    fn test_login() {
        let username = std::env::var("TEST_USERNAME").unwrap();
        let password = std::env::var("TEST_PASSWORD").unwrap();
        super::login_and_print(username, password);
        super::logout();
    }
}
