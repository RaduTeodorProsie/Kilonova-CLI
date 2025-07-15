use crate::cli::{Cli, Commands};
use clap::Parser;
use reqwest::blocking;
use std::{env, io};

mod cli;
mod credential_manager;

fn main() {
    let cli = Cli::parse();
    match cli.command {
        Commands::Login => {}
        Commands::Logout => {}
        Commands::Me => {}
        Commands::Search { .. } => {}
        Commands::Submit { .. } => {}
        Commands::Start => {}
    }
}
