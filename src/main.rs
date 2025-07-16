use crate::cli::{Cli, Commands};
use clap::Parser;
use reqwest::blocking;
use std::{env, io};

mod cli;
mod credential_manager;
mod logging;
mod waiter;

fn main() {
    let cli = Cli::parse();
    match cli.command {
        Commands::Login => {
            logging::login();
        }
        Commands::Logout => {
            logging::logout();
        }
        Commands::Me => {}
        Commands::Search { .. } => {}
        Commands::Submit { .. } => {}
        Commands::Start => {}
    }
}
