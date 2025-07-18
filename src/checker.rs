use super::*;
use colored::Colorize;

pub fn setup() {
    let spinner = waiter::Waiter::start();
    let response = blocking::Client::new()
        .get("https://kilonova.ro/")
        .timeout(std::time::Duration::from_secs(10))
        .send()
        .expect("Error during request, kn might have timed out");
    let kn_status = match response.status() {
        reqwest::StatusCode::OK => "Kn is reachable ✅".green(),
        reqwest::StatusCode::NOT_FOUND => "Kn is unreachable ❌".red(),
        _ => "Unknown error".bright_yellow(),
    };

    spinner.stop();
    let user_info = user_info::get_user();
    let user_info = match user_info {
        Ok(user) => {
            match logging::extend_session() {
                Ok(_) => println!(
                    "{}",
                    "Session successfully extended for another 30 days".green()
                ),
                Err(e) => println!("{} : {}", "Could not extend session".yellow(), e),
            }
            format!("{}", format!("Logged in as {} ✅", user).green())
        }
        Err(_) => "You are not logged in.".yellow().to_string(),
    };

    println!("{}", kn_status);
    println!("{}", user_info);
}

mod tests {

    #[test]
    fn prints_kn_and_user_info() {
        super::setup();
    }
}
