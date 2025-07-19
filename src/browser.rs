use base64::Engine;
use base64::engine::general_purpose;
use crossterm::style::Print;
use crossterm::{
    cursor,
    event::{self, Event, KeyCode, KeyModifiers},
    execute,
    style::{Attribute, SetAttribute},
    terminal::{self, Clear, ClearType, disable_raw_mode, enable_raw_mode},
};
use super::credential_manager;
use reqwest::blocking::Client;
use scraper::{Html, Selector};
use serde::Deserialize;
use std::io::{Stdout, Write, stdout};
use std::time::{Duration, Instant};

use termimad::print_text;


#[derive(Deserialize, Debug)]
#[serde(rename_all = "snake_case")]
struct AttachmentData {
    metadata: serde_json::Value,
    mime_type: String,
    data: String,
}

#[derive(Deserialize, Debug)]
struct ApiResponse {
    status: String,
    data: AttachmentData,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct ProblemSummary {
    pub id: u64,
    pub name: String,
}

fn fetch_summaries(
    client: &Client,
    query: &str,
    page: u8,
) -> Result<Vec<ProblemSummary>, Box<dyn std::error::Error>> {
    let response = client
        .get("https://kilonova.ro/problems")
        .query(&[("q", query), ("page", &page.to_string())])
        .send()?;
    let body = response.text()?;
    let document = Html::parse_document(&body);
    let selector = Selector::parse("kn-pb-search[enc]").unwrap();
    let elem = document
        .select(&selector)
        .next()
        .ok_or("No <kn-pb-search enc> element found")?;
    let enc = elem.value().attr("enc").ok_or("Missing `enc` attribute")?;
    let raw = general_purpose::STANDARD.decode(enc)?;
    let summaries = serde_json::from_slice(&raw)?;
    Ok(summaries)
}

fn draw_line(out: &mut Stdout, text: &str, row: u16, is_selected: bool) {
    execute!(out, cursor::MoveTo(0, row), Clear(ClearType::CurrentLine)).unwrap();
    let prefix = if is_selected { "> " } else { "  " };
    if is_selected {
        execute!(out, SetAttribute(Attribute::Reverse)).unwrap();
    }
    execute!(out, Print(format!("{}{}", prefix, text))).unwrap();
    if is_selected {
        execute!(out, SetAttribute(Attribute::Reset)).unwrap();
    }
}

fn choose_from_list(out: &mut Stdout, v: &[ProblemSummary]) -> String {
    while event::poll(Duration::from_millis(0)).unwrap_or(false) {
        let _ = event::read();
    }

    if v.is_empty() {
        execute!(out, cursor::MoveTo(0, 0), Print("No results found on this page.")).unwrap();
        execute!(out, cursor::MoveTo(0, 2), Print("Press k for back, or Esc/q to quit.")).unwrap();
        out.flush().unwrap();
        loop {
            if let Event::Key(key) = event::read().expect("Failed to read event") {
                match key.code {
                    KeyCode::Left | KeyCode::Char('k') => return "left".into(),
                    KeyCode::Esc | KeyCode::Char('q') => return "esc".into(),
                    KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => return "esc".into(),
                    _ => {}
                }
            }
        }
    }

    let mut selected_idx: usize = 0;
    let mut last_input_time = Instant::now();
    let cooldown = Duration::from_millis(100);

    for (i, p) in v.iter().enumerate() {
        draw_line(out, &p.name, i as u16, i == selected_idx);
    }
    out.flush().unwrap();

    loop {
        if let Event::Key(key) = event::read().expect("Failed to read event") {
            if last_input_time.elapsed() >= cooldown {
                let prev_idx = selected_idx;
                match key.code {
                    KeyCode::Up | KeyCode::Char('f') => if selected_idx > 0 { selected_idx -= 1; },
                    KeyCode::Down | KeyCode::Char('d') => if selected_idx + 1 < v.len() { selected_idx += 1; },
                    KeyCode::Left | KeyCode::Char('k') => return "left".into(),
                    KeyCode::Right | KeyCode::Char('j') => return "right".into(),
                    KeyCode::Enter => return v[selected_idx].id.to_string(),
                    KeyCode::Esc | KeyCode::Char('q') => return "esc".into(),
                    KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => return "esc".into(),
                    _ => {}
                }
                last_input_time = Instant::now();
                if selected_idx != prev_idx {
                    draw_line(out, &v[prev_idx].name, prev_idx as u16, false);
                    draw_line(out, &v[selected_idx].name, selected_idx as u16, true);
                    out.flush().unwrap();
                }
            }
        }
    }
}

fn display_problem(id: &str) {
    credential_manager::CredentialManager::global()
        .set::<credential_manager::Cache>(id)
        .expect("Couldn't save the id token");

    let client = Client::new();
    let statement_files = ["statement-ro.md", "statement-en.md"];

    println!("Searching for problem statement for ID: {}...", id);

    let statement_content: Option<String> = statement_files.iter().find_map(|&file_name| {
        let fetch_and_decode = || -> Result<String, Box<dyn std::error::Error>> {
            let url = format!("https://kilonova.ro/api/problem/{}/get/attachmentByName/{}", id, file_name);
            let response = client.get(url).send()?;
            let api_response = serde_json::from_str::<ApiResponse>(response.text()?.as_str())?;
            let decoded_bytes = general_purpose::STANDARD.decode(api_response.data.data)?;
            let content = String::from_utf8(decoded_bytes)?;
            Ok(content)
        };

        fetch_and_decode().ok()
    });

    if let Some(content) = statement_content {
        println!("\nSuccessfully fetched and decoded statement:\n---");

        print_text(&content);
        println!("---");
    } else {
        println!("\nCould not find a valid, readable statement for problem ID: {}", id);
    }
}

pub fn search(name: &str) {
    let client = Client::new();
    let mut page: u8 = 1;

    let mut out = stdout();
    enable_raw_mode().expect("Failed to enable raw mode");
    execute!(out, cursor::Hide).expect("Failed to hide cursor");

    let final_choice = loop {
        execute!(out, Clear(ClearType::All), cursor::MoveTo(0, 0)).expect("Failed to clear screen");

        let summaries = match fetch_summaries(&client, name, page) {
            Ok(s) => s,
            Err(_) => {
                if page > 1 { page -= 1; }
                continue;
            }
        };

        if summaries.is_empty() && page > 1 {
            page = page.saturating_sub(1);
            continue;
        }

        let choice = choose_from_list(&mut out, &summaries);

        match choice.as_ref() {
            "right" => {
                page += 1;
                continue;
            }
            "left" => {
                if page > 1 { page -= 1; }
                continue;
            }
            _ => break choice,
        }
    };

    execute!(out, Clear(ClearType::All), cursor::MoveTo(0, 0)).unwrap();
    execute!(out, cursor::Show).expect("Failed to show cursor");
    disable_raw_mode().expect("Failed to disable raw mode");

    if final_choice != "esc" {
        display_problem(&final_choice);
    }
}