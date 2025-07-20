use clap::{Parser, Subcommand};
use std::ffi::OsString;

#[derive(Debug, Parser)]
#[command(
    about = "A cli tool for interacting with the kilonova API",
    version = "2.0"
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    #[clap(about = "Check the status of the API and extend your login session token")]
    Start,

    #[clap(about = "Login to the platform")]
    Login,

    #[clap(about = "Logout from the platform")]
    Logout,

    #[clap(about = "Get your user information")]
    Me,

    #[clap(about = "Search for a problem by name")]
    Search { name: String },

    #[clap(about = "Submit a solution to the last viewed problem")]
    Submit { path: OsString },

    #[clap(about = "Set the default language for submissions")]
    SetLanguage { name: String },

    #[clap(about = "Set the default language for statements")]
    SetStatementLanguage { name: String },

    #[clap(about = "View the last seen problem statement")]
    View,
}
