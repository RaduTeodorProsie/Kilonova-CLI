use reqwest::blocking::Client;
use serde_json::Value;
use std::io;
use std::io::Write;
use colored::Colorize;
use crate::credential_manager::CredentialManager;

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

fn internal_login(username: String, password: String) {
    let client = Client::new();
    let response_text = client
        .post("https://kilonova.ro/api/auth/login")
        .query(&[("username", username), ("password", password)])
        .send()
        .expect("Could not send the reqwest")
        .text()
        .unwrap();

    let response: Value = serde_json::from_str(&response_text).unwrap();
    let status = response["status"].as_str().unwrap();
    let data   = response["data"].as_str().unwrap();

    match status {
        "success" => {
            println!("{}", "Successfully logged in ✅".green());
            use super::credential_manager;
            CredentialManager::global().set::<credential_manager::Cache>(data).expect("Could not cache the token");
        }
        _ => {
            println!("{}", "Wrong credentials".red());
        }
    }

}

pub fn login() {
    let (username, password) = read_username_and_password();
    internal_login(username, password);
}

mod tests {
    #[test]
    fn test_login() {
        super::internal_login("tester".to_string(), "test123".to_string());
    }
}
