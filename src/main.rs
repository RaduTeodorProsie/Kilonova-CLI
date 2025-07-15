

use std::io;
use clap::Parser;
use indicatif;
use colored::Colorize;
use reqwest::blocking;
use crate::cli::{Cli, Commands};

mod cli;
mod credential_manager;

fn main() {
    let cli = Cli::parse();
    match cli.command {
        Commands::Login => {}
        Commands::Logout => {}
        Commands::Search { .. } => {}
        Commands::Submit { .. } => {},
        Commands::Start => {}
    }
}
