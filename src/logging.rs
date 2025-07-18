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

fn internal_login(username: String, password: String) -> Result<String, reqwest::Error> {
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
            return ();
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

pub fn extend_session() -> Result<(), String> {
    let token = CredentialManager::global()
        .get::<credential_manager::Token>()
        .unwrap();

    let resp = Client::new()
        .post("https://kilonova.ro/api/auth/extendSession")
        .header("Authorization", token)
        .timeout(Duration::from_secs(10))
        .send()
        .map_err(|e| e.to_string())?;

    if resp.status() != reqwest::StatusCode::OK {
        return Err(resp.text().map_err(|e| e.to_string())?);
    }

    Ok(())
}

pub fn logout() {
    // 1) start the spinner
    let mut waiter = Waiter::start();

    // 2) grab the token (or bail)
    let token =
        match credential_manager::CredentialManager::global().get::<credential_manager::Token>() {
            Some(tok) => tok,
            None => {
                waiter.stop();
                eprintln!("{}", "Error: Not logged in.".red());
                return;
            }
        };

    if let Err(e) = Client::new()
        .post("https://kilonova.ro/api/auth/logout")
        .header("Authorization", token)
        .timeout(Duration::from_secs(10))
        .send()
    {
        waiter.stop();
        eprintln!("{} {}", "Error: Could not send logout request:".red(), e);
        return;
    }

    if let Err(e) = CredentialManager::global().delete::<credential_manager::Token>() {
        waiter.stop();
        eprintln!("{} {}", "Error: Could not delete token:".red(), e);
        return;
    }

    waiter.stop();
    println!("{}", "Successfully logged out ✅".green());
}

#[cfg(test)]
pub mod tests {
    use super::*;

    #[test]
    pub fn test_login() {
        let username = std::env::var("TEST_USERNAME").unwrap();
        let password = std::env::var("TEST_PASSWORD").unwrap();

        login_and_print(username, password);
        extend_session().unwrap_or_else(|e| panic!("{}", e));
        logout();
    }
}
