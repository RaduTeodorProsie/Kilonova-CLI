use super::*;
use crate::submitter::Range::{Double, Single};
use colored::Colorize;
use serde::Deserialize;
use serde_json;
use serde_json::{Value, from_str};
use std::collections::HashMap;
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
    problem: Problem,
    subtests: Vec<TestInfo>,
}

#[derive(Deserialize)]
struct Problem {
    time_limit: f64,
    memory_limit: u64,
}

#[derive(Deserialize)]
struct TestInfo {
    visible_id: u16,
    skipped: bool,
    time: f64,
    memory: u64,
    percentage: f64,
}

fn get_status(test: &TestInfo) -> String {
    if test.skipped == true {
        return "skipped".to_string();
    }

    if test.percentage == 100.0 {
        return "correct".to_string();
    }

    if test.percentage == 0.0 {
        return "wrong answer".to_string();
    }

    "partially correct".to_string()
}

enum Range {
    Single(u16),
    Double(u16, u16),
}

fn split_in_ranges(v: &Vec<u16>) -> Vec<Range> {
    if v.is_empty() {
        return Vec::<Range>::new();
    }

    let mut ans: Vec<Range> = vec![];
    let mut range = Single(v[0]);
    for &curr in &v[1..] {
        range = match range {
            Single(prev) if prev + 1 == curr => Double(prev, curr),
            Double(first, second) if second + 1 == curr => Double(first, curr),
            prev => {
                ans.push(prev);
                Single(curr)
            }
        };
    }

    ans.push(range);
    ans
}

fn print_result(json: ApiResponse) {
    let test_results = json.data.subtests;
    let problem = json.data.problem;
    let mut hashmap: HashMap<String, Vec<u16>> = HashMap::new();
    for t in test_results {
        if t.time == problem.time_limit {
            hashmap
                .entry("TLE".to_string())
                .or_insert(vec![])
                .push(t.visible_id);
        } else if t.memory == problem.memory_limit {
            hashmap
                .entry("MLE".to_string())
                .or_insert(vec![])
                .push(t.visible_id);
        } else if t.skipped == true {
            hashmap
                .entry("skipped".to_string())
                .or_insert(vec![])
                .push(t.visible_id);
        } else {
            hashmap
                .entry(get_status(&t))
                .or_insert(vec![])
                .push(t.visible_id);
        }
    }

    let verdicts = [
        "correct".green(),
        "wrong answer".red(),
        "partially correct".yellow(),
        "TLE".red(),
        "MLE".red(),
        "skipped".white(),
    ];

    for verdict in verdicts {
        let ranges = split_in_ranges(hashmap.entry((&verdict).parse().unwrap()).or_insert(vec![]));
        if ranges.is_empty() {
            continue;
        }

        let mut msg = verdict.clone();
        msg.input = "Verdict :".to_string();
        let mut msg2 = verdict.clone();
        msg2.input = "on ".to_string();

        print!("{} {} {}", msg, verdict.clone(), msg2);

        for range in ranges {
            let mut msg = verdict.clone();
            msg.input = match range {
                Single(x) => format!("test {}", x),
                Double(first, last) => format!("tests {} through {}; ", first, last),
            };

            print!("{msg}");
        }

        println!();
    }

    if json.data.score == 100.0 {
        println!("{} {}", "Total score: ".green(), json.data.score.floor());
    } else if json.data.score == 0.0 {
        println!("{} {}", "Total score: ".red(), json.data.score.floor());
    } else {
        println!("{} {}", "Total score: ".yellow(), json.data.score.floor());
    }
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
        thread::sleep(POLL_INTERVAL);
        match reqwest::blocking::Client::new().get(&api_url).send() {
            Ok(response) if response.status() == reqwest::StatusCode::OK => {
                let json = response.text().unwrap();
                let json: ApiResponse = serde_json::from_str(&json)
                    .expect("Couldn't parse the response from the server");
                let status: &str = json.data.status.as_ref();
                if status == "finished" {
                    spinner.stop();
                    print_result(json);
                    break;
                }
            }

            Ok(_) => continue,

            Err(e) => {
                spinner.stop();
                eprintln!("Request failed: {}", e);
                break;
            }
        }
    }
}
