use crate::{credential_manager, waiter};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct ApiResponse {
    data: UserData,
}

#[derive(Debug, Deserialize)]
struct UserData {
    name: String,
}

pub fn get_user() -> Result<String, String> {
    let token = credential_manager::CredentialManager::global().get::<credential_manager::Token>();
    if token.is_none() {
        return Err(String::from("You need to login before doing this."));
    }

    let token = token.unwrap();

    let resp = reqwest::blocking::Client::new()
        .get("https://kilonova.ro/api/user/self")
        .header("Authorization", token)
        .timeout(std::time::Duration::from_secs(10))
        .send()
        .map_err(|e| e.to_string())?
        .text()
        .map_err(|e| e.to_string())?;
    let api_response = serde_json::from_str::<ApiResponse>(&resp);
    let api_response = api_response.map(|api_response| api_response.data.name);
    api_response.map_err(|e| e.to_string())
}

pub fn get() {
    let spinner = waiter::Waiter::start();
    let name = get_user();
    let print = match name {
        Ok(name) => format!("Logged in as {}", name),
        Err(e) => format!("Error : {}", e.to_string()),
    };

    spinner.stop();
    println!("{}", print);
}
