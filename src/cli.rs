use std::path::PathBuf;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(
    name = "drill",
    about = "Daily exercise rotation — no repeats, full coverage"
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand)]
pub enum Command {
    /// Add a new exercise unit
    Add {
        name: String,
        #[arg(short, long)]
        path: PathBuf,
        #[arg(short, long, default_value = "ers")]
        ext: String,
        #[arg(short, long, default_value_t = 4)]
        count: usize,
    },
    /// Remove an exercise unit
    Remove { name: String },
    /// List configured units
    List,
    /// Propose today's exercises
    Propose,
    /// Manage blacklisted files
    Blacklist {
        #[command(subcommand)]
        action: BlacklistAction,
    },
    /// Show recent history
    History {
        #[arg(long, default_value_t = 7)]
        days: u32,
    },
    /// Pick extra exercises from a unit (on top of today's set)
    More {
        unit: String,
        #[arg(default_value_t = 4)]
        count: usize,
        /// Skip confirmation prompt
        #[arg(short, long)]
        yes: bool,
    },
    /// Mark an exercise as failed (retry tomorrow)
    Fail { unit: String, file: String },
    /// Mark a failed exercise as passed (remove from retries)
    Pass { unit: String, file: String },
    /// Show per-unit statistics
    Status,
    /// Reset cycle for a unit (or all units)
    Reset { unit: Option<String> },
}

#[derive(Subcommand)]
pub enum BlacklistAction {
    /// Add a file to the blacklist
    Add { unit: String, file: String },
    /// Remove a file from the blacklist
    Remove { unit: String, file: String },
    /// List blacklisted files
    List { unit: Option<String> },
}
