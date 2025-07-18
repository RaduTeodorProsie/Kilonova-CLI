use super::*;
use colored::Colorize;
use serde::Deserialize;
use serde_json;
use serde_json::Value;
use std::ffi::OsString;
use std::path::PathBuf;
use std::thread;
use std::time::Duration;

#[derive(Deserialize)]
struct ApiResponse {
    data: SubmissionData,
}

#[derive(Deserialize)]
struct SubmissionData {
    status: String,
    score: f64,
}

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

    let json: Value = serde_json::from_str(&response.text().unwrap())
        .expect("Couldn't parse the response from the server");
    let submission_id = json["data"].as_u64().unwrap();
    let submission_id = submission_id.to_string();

    println!("{}", "Submitted your code. Judging...".green());

    let spinner = waiter::Waiter::start();
    const POLL_INTERVAL: Duration = Duration::from_secs(5);
    let api_url = format!(
        "https://kilonova.ro/api/submissions/getByID?id={}",
        submission_id
    );

    loop {
        match reqwest::blocking::Client::new().get(&api_url).send() {
            Ok(response) => {
                let json = response.text().unwrap();
                let json: ApiResponse = serde_json::from_str(&json)
                    .expect("Couldn't parse the response from the server");
                let (status, score): (&str, f64) = (json.data.status.as_ref(), json.data.score);
                if status == "finished" {
                    spinner.stop();

                    let score = score.floor() as i64;
                    if score == 100 {
                        println!("{} {}", "Your score is : ".green(), score);
                    } else if score <= 50 {
                        println!("{} {}", "Your score is : ".red(), score);
                    } else {
                        println!("{} {}", "Your score is : ".yellow(), score);
                    }

                    break;
                }
            }

            Err(e) => {
                spinner.stop();
                eprintln!("Request failed: {}", e);
                break;
            }
        }

        thread::sleep(POLL_INTERVAL);
    }
}
