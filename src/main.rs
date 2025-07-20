use crate::cli::{Cli, Commands};
use clap::Parser;
use reqwest::blocking;

mod cli;
mod credential_manager;
mod logging;
mod waiter;

mod checker;

mod browser;
mod language;
mod statement;
mod submitter;
mod user_info;
mod view;

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Login => {
            logging::login();
        }

        Commands::Logout => {
            logging::logout();
        }

        Commands::Me => user_info::get(),
        Commands::Search { name } => browser::search(&name),
        Commands::Submit { path } => {
            submitter::submit(path);
        }
        Commands::SetLanguage { name } => {
            language::set_language(name.as_ref());
        }
        Commands::Start => {
            checker::setup();
        }

        Commands::SetStatementLanguage { name } => {
            statement::set_language(name.as_ref());
        }

        Commands::View => {
            view::view_latest_statement();
        }
    }
}
