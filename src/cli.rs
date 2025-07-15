use std::ffi::OsString;
use clap::{Parser, Subcommand};

#[derive (Debug, Parser)]
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
    Search {name: String},
    Submit {path: OsString},
}