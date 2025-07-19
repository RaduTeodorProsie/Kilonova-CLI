use clap::{Parser, Subcommand};
use std::ffi::OsString;

#[derive(Debug, Parser)]
#[command(name = "kn")]
#[command(about = "A simple CLI tool", version = "0.1")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    Start,
    Login,
    Logout,
    Me,
    Search { name: String },
    Submit { path: OsString },
    SetLanguage { name: String },
}
