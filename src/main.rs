mod cli;
mod commands;
mod config;
mod display;
mod error;
mod proposer;
mod scanner;
mod state;

use clap::Parser;
use cli::{BlacklistAction, Cli, Command};

fn main() {
    if let Err(e) = run() {
        eprintln!("error: {e}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), error::DrillError> {
    let cli = Cli::parse();
    let mut config = config::Config::load()?;
    let mut state = state::State::load()?;

    match cli.command {
        Command::Add {
            name,
            path,
            ext,
            count,
        } => commands::add(&mut config, &name, path, &ext, count),
        Command::Remove { name } => commands::remove(&mut config, &mut state, &name),
        Command::List => commands::list(&config),
        Command::Propose => commands::propose(&config, &mut state),
        Command::Blacklist { action } => match action {
            BlacklistAction::Add { unit, file } => {
                commands::blacklist_add(&mut config, &mut state, &unit, &file)
            }
            BlacklistAction::Remove { unit, file } => {
                commands::blacklist_remove(&mut config, &unit, &file)
            }
            BlacklistAction::List { unit } => commands::blacklist_list(&config, unit.as_deref()),
        },
        Command::More { unit, count, yes } => {
            commands::more(&config, &mut state, &unit, count, yes)
        }
        Command::Fail { unit, file } => commands::fail(&config, &mut state, &unit, &file),
        Command::Pass { unit, file } => commands::pass(&config, &mut state, &unit, &file),
        Command::Set { unit, count } => commands::set_count(&mut config, &unit, count),
        Command::History { days } => commands::history(&state, days),
        Command::Status => commands::status(&config, &state),
        Command::Reset { unit } => commands::reset(&config, &mut state, unit.as_deref()),
    }
}
