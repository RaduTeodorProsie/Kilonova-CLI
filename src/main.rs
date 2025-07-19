use crate::cli::{Cli, Commands};
use clap::Parser;
use reqwest::blocking;
use std::{env, io};

mod cli;
mod credential_manager;
mod logging;
mod waiter;

mod checker;

mod browser;
mod language;
mod submitter;
mod user_info;

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
    }
}
