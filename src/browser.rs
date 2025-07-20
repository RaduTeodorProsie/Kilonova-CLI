use super::credential_manager;
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
use reqwest::blocking::Client;
use scraper::{Html, Selector};
use serde::Deserialize;
use std::io::{Stdout, Write, stdout};
use std::time::{Duration, Instant};

#[derive(Deserialize, Debug)]
#[serde(rename_all = "snake_case")]
struct AttachmentData {
    data: String,
}

#[derive(Deserialize, Debug)]
struct ApiResponse {
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
        execute!(
            out,
            cursor::MoveTo(0, 0),
            Print("No results found on this page.")
        )
        .unwrap();
        execute!(
            out,
            cursor::MoveTo(0, 2),
            Print("Press k for back, or Esc/q to quit.")
        )
        .unwrap();
        out.flush().unwrap();
        loop {
            if let Event::Key(key) = event::read().expect("Failed to read event") {
                match key.code {
                    KeyCode::Left | KeyCode::Char('k') => return "left".into(),
                    KeyCode::Esc | KeyCode::Char('q') => return "esc".into(),
                    KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                        return "esc".into();
                    }
                    _ => {}
                }
            }
        }
    }

    let instructions = "Controls: [f/d] scroll up/down | [j/k] next/previous page, [q] to quit";
    execute!(out, cursor::MoveTo(0, 0), Print(instructions)).unwrap();

    let mut selected_idx: usize = 0;
    let mut last_input_time = Instant::now();
    let cooldown = Duration::from_millis(100);

    for (i, p) in v.iter().enumerate() {
        // Offset drawing by 2 rows to make space for instructions and a blank line
        draw_line(out, &p.name, (i + 2) as u16, i == selected_idx);
    }
    out.flush().unwrap();

    loop {
        if let Event::Key(key) = event::read().expect("Failed to read event") {
            if last_input_time.elapsed() >= cooldown {
                let prev_idx = selected_idx;
                match key.code {
                    KeyCode::Up | KeyCode::Char('f') => {
                        if selected_idx > 0 {
                            selected_idx -= 1;
                        }
                    }
                    KeyCode::Down | KeyCode::Char('d') => {
                        if selected_idx + 1 < v.len() {
                            selected_idx += 1;
                        }
                    }
                    KeyCode::Left | KeyCode::Char('k') => return "left".into(),
                    KeyCode::Right | KeyCode::Char('j') => return "right".into(),
                    KeyCode::Enter => return v[selected_idx].id.to_string(),
                    KeyCode::Esc | KeyCode::Char('q') => return "esc".into(),
                    KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                        return "esc".into();
                    }
                    _ => {}
                }
                last_input_time = Instant::now();
                if selected_idx != prev_idx {
                    // Also offset these drawing calls by 2 rows
                    draw_line(out, &v[prev_idx].name, (prev_idx + 2) as u16, false);
                    draw_line(out, &v[selected_idx].name, (selected_idx + 2) as u16, true);
                    out.flush().unwrap();
                }
            }
        }
    }
}

struct Pager<'a> {
    lines: Vec<&'a str>,
    top_line: usize,
    start_row: u16,
    view_height: u16,
    should_quit: bool,
}

impl<'a> Pager<'a> {
    fn new(content: &'a str) -> Self {
        Self {
            lines: content.lines().collect(),
            top_line: 0,
            start_row: 0,   // Will be set at runtime
            view_height: 0, // Will be set at runtime
            should_quit: false,
        }
    }

    fn run(&mut self) -> Result<(), std::io::Error> {
        let mut out = stdout();
        enable_raw_mode()?;

        self.start_row = cursor::position()?.1;
        let (_, term_height) = terminal::size()?;
        self.view_height = term_height.saturating_sub(self.start_row).max(1);

        while !self.should_quit {
            self.render(&mut out)?;
            self.handle_input()?;
        }

        self.render(&mut out)?;
        let final_row = self.start_row
            + self
                .lines
                .iter()
                .skip(self.top_line)
                .take(self.view_height as usize)
                .count() as u16;
        execute!(out, cursor::MoveTo(0, final_row), Print("\n"))?;

        disable_raw_mode()
    }

    fn render(&self, out: &mut Stdout) -> Result<(), std::io::Error> {
        execute!(out, cursor::SavePosition)?;
        execute!(
            out,
            cursor::MoveTo(0, self.start_row),
            Clear(ClearType::FromCursorDown)
        )?;

        let visible_lines = self
            .lines
            .iter()
            .skip(self.top_line)
            .take(self.view_height as usize);
        for (i, line) in visible_lines.enumerate() {
            execute!(
                out,
                cursor::MoveTo(0, self.start_row + i as u16),
                Print(line)
            )?;
        }
        execute!(out, cursor::RestorePosition)?;
        out.flush()
    }

    fn handle_input(&mut self) -> Result<(), std::io::Error> {
        if let Event::Key(key) = event::read()? {
            match key.code {
                KeyCode::Char('q') | KeyCode::Esc => self.should_quit = true,
                KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                    self.should_quit = true;
                }
                KeyCode::Up | KeyCode::Char('k') => {
                    self.top_line = self.top_line.saturating_sub(1);
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    let max_top_line = self.lines.len().saturating_sub(self.view_height as usize);
                    if self.top_line < max_top_line {
                        self.top_line += 1;
                    }
                }
                KeyCode::PageUp => {
                    self.top_line = self.top_line.saturating_sub(self.view_height as usize);
                }
                KeyCode::PageDown | KeyCode::Char(' ') => {
                    let max_top_line = self.lines.len().saturating_sub(self.view_height as usize);
                    let new_top = self.top_line.saturating_add(self.view_height as usize);
                    self.top_line = new_top.min(max_top_line);
                }
                _ => {}
            }
        }
        Ok(())
    }
}

pub(crate) fn display_problem(id: &str) {
    credential_manager::CredentialManager::global()
        .set::<credential_manager::Cache>(id)
        .expect("Couldn't save the id token");

    let client = Client::new();
    let statement_files = ["statement-ro.md", "statement-en.md"];

    println!("Searching for problem statement for ID: {}...", id);

    let statement_content: Option<String> = statement_files.iter().find_map(|&file_name| {
        let fetch_and_decode = || -> Result<String, Box<dyn std::error::Error>> {
            let url = format!(
                "https://kilonova.ro/api/problem/{}/get/attachmentByName/{}",
                id, file_name
            );
            let response = client.get(url).send()?;
            if !response.status().is_success() {
                return Err("Failed to fetch from API".into());
            }
            let text = response.text()?;
            if text.trim().is_empty() {
                return Err("Empty response".into());
            }
            let api_response: ApiResponse = serde_json::from_str(&text).map_err(|e| {
                format!(
                    "JSON parsing error: '{}' on text starting with: '{}'",
                    e,
                    text.chars().take(100).collect::<String>()
                )
            })?;
            let decoded_bytes = general_purpose::STANDARD.decode(api_response.data.data)?;
            String::from_utf8(decoded_bytes).map_err(|e| e.into())
        };
        fetch_and_decode().ok()
    });

    if let Some(content) = statement_content {
        println!("\n(use j/k to scroll, q to exit)");

        let mut pager = Pager::new(&content);
        if let Err(e) = pager.run() {
            let _ = disable_raw_mode();
            eprintln!("\nPager Error: {}", e);
        }
    } else {
        println!(
            "\nCould not find a valid, readable statement for problem ID: {}",
            id
        );
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
                if page > 1 {
                    page = page.saturating_sub(1);
                }
                continue;
            }
        };

        if summaries.is_empty() && page > 1 {
            page = page.saturating_sub(1);
            continue;
        }

        let choice = choose_from_list(&mut out, &summaries);

        match choice.as_str() {
            "right" => {
                page += 1;
                continue;
            }
            "left" => {
                if page > 1 {
                    page = page.saturating_sub(1);
                }
                continue;
            }
            _ => break choice,
        }
    };

    execute!(
        out,
        Clear(ClearType::All),
        cursor::MoveTo(0, 0),
        cursor::Show
    )
    .unwrap();
    disable_raw_mode().expect("Failed to disable raw mode");

    if final_choice != "esc" {
        display_problem(&final_choice);
    }
}
