use super::*;
use colored::Colorize;
use std::ffi::OsString;
use std::path::PathBuf;
use std::time::Duration;

pub fn submit(path: OsString) {
    let problem =
        credential_manager::CredentialManager::global().get::<credential_manager::Cache>();
    let problem = match problem {
        Some(p) => p,
        None => {
            println!("{}", "Submissions are made to the last seen problem. You need to look at a problem first.".bright_yellow());
            return;
        }
    };

    let token = credential_manager::CredentialManager::global().get::<credential_manager::Token>();
    let token = match token {
        Some(t) => t,
        None => {
            println!(
                "{}",
                "You need to be logged in before you can submit".bright_yellow()
            );
            return;
        }
    };

    let spinner = waiter::Waiter::start();
    let file_path: PathBuf = path.into();
    let form = reqwest::blocking::multipart::Form::new()
        .text("problem_id", problem)
        .text("language", "rust")
        .part(
            "code",
            reqwest::blocking::multipart::Part::file(&file_path)
                .expect("Could not open source file")
                .mime_str("text/plain")
                .expect("Invalid MIME"),
        );

    let response = reqwest::blocking::Client::new()
        .post("https://kilonova.ro/api/submissions/submit")
        .header("Authorization", token)
        .timeout(Duration::from_secs(30))
        .multipart(form)
        .send();

    spinner.stop();
    let response = match response {
        Ok(r) => r,
        Err(e) => {
            println!("{} {}", "Couldn't submit your code : ".red(), e);
            return;
        }
    };

    if response.status() != reqwest::StatusCode::OK {
        println!(
            "{} {}",
            "Couldn't submit your code : ".red(),
            response.text().unwrap()
        );
        return;
    }

    println!("{}", "Submitted your code. Judging...".green());
    let spinner = waiter::Waiter::start();
    const POLL_INTERVAL: Duration = Duration::from_secs(5);
    spinner.stop();
}
