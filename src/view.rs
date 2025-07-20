use super::*;
use colored::Colorize;

use crossterm::{
    cursor, execute,
    terminal::{Clear, ClearType, disable_raw_mode, enable_raw_mode},
};
use std::io::stdout;

pub fn view_latest_statement() {
    let problem_id =
        credential_manager::CredentialManager::global().get::<credential_manager::Cache>();

    if let Some(id) = problem_id {
        execute!(stdout(), Clear(ClearType::All), cursor::MoveTo(0, 0)).unwrap();

        disable_raw_mode().unwrap();
        execute!(stdout(), cursor::Show).unwrap();

        browser::display_problem(&id);

        enable_raw_mode().unwrap();
        execute!(stdout(), cursor::Hide).unwrap();
    } else {
        println!(
            "{}",
            "You need to look at a problem before doing this".yellow()
        );
    }
}
