use std::{
    io::{self, Write},
    path::PathBuf,
};

use chrono::{Days, Local};

use crate::{
    config::{Config, UnitConfig},
    display,
    error::DrillError,
    proposer, scanner,
    state::State,
};

pub fn add(
    config: &mut Config,
    name: &str,
    path: PathBuf,
    ext: &str,
    count: usize,
) -> Result<(), DrillError> {
    if config.units.contains_key(name) {
        return Err(DrillError::UnitAlreadyExists(name.to_string()));
    }
    if !path.exists() {
        return Err(DrillError::PathNotFound(path));
    }

    let files = scanner::scan_exercises(&path, ext)?;
    if files.is_empty() {
        return Err(DrillError::NoExercisesFound {
            path,
            ext: ext.to_string(),
        });
    }

    config.units.insert(
        name.to_string(),
        UnitConfig {
            path,
            extension: ext.to_string(),
            count,
            blacklist: Vec::new(),
        },
    );
    config.save()?;

    println!("Added unit '{name}': {} exercises found", files.len());
    Ok(())
}

pub fn remove(config: &mut Config, state: &mut State, name: &str) -> Result<(), DrillError> {
    if !config.units.contains_key(name) {
        return Err(DrillError::UnitNotFound(name.to_string()));
    }

    config.units.remove(name);
    state.cycles.remove(name);
    state.retries.remove(name);
    config.save()?;
    state.save()?;

    println!("Removed unit '{name}'");
    Ok(())
}

pub fn list(config: &Config) -> Result<(), DrillError> {
    if config.units.is_empty() {
        println!("No units configured. Use 'drill add' to add one.");
        return Ok(());
    }

    for (name, unit) in &config.units {
        let total = scanner::scan_exercises(&unit.path, &unit.extension)
            .map(|f| f.len())
            .unwrap_or(0);
        println!(
            "  {name} — {total} files, pick {}, {} blacklisted",
            unit.count,
            unit.blacklist.len()
        );
        println!("    path: {}", unit.path.display());
    }
    Ok(())
}

pub fn propose(config: &Config, state: &mut State) -> Result<(), DrillError> {
    if config.units.is_empty() {
        println!("No units configured. Use 'drill add' to add one.");
        return Ok(());
    }

    let (session, is_existing) = proposer::propose(config, state)?;
    display::print_session(&session, &state.retries, is_existing);
    Ok(())
}

pub fn blacklist_add(
    config: &mut Config,
    state: &mut State,
    unit: &str,
    file: &str,
) -> Result<(), DrillError> {
    let unit_config = config
        .units
        .get_mut(unit)
        .ok_or_else(|| DrillError::UnitNotFound(unit.to_string()))?;

    let files = scanner::scan_exercises(&unit_config.path, &unit_config.extension)?;
    if !files.contains(&file.to_string()) {
        return Err(DrillError::FileNotInUnit {
            unit: unit.to_string(),
            file: file.to_string(),
        });
    }

    if unit_config.blacklist.contains(&file.to_string()) {
        return Err(DrillError::AlreadyBlacklisted {
            unit: unit.to_string(),
            file: file.to_string(),
        });
    }

    unit_config.blacklist.push(file.to_string());
    unit_config.blacklist.sort();

    // Remove from cycle tracking if present
    if let Some(cycle) = state.cycles.get_mut(unit) {
        cycle.proposed.retain(|f| f != file);
    }

    config.save()?;
    state.save()?;

    println!("Blacklisted '{file}' in unit '{unit}'");
    Ok(())
}

pub fn blacklist_remove(config: &mut Config, unit: &str, file: &str) -> Result<(), DrillError> {
    let unit_config = config
        .units
        .get_mut(unit)
        .ok_or_else(|| DrillError::UnitNotFound(unit.to_string()))?;

    if !unit_config.blacklist.contains(&file.to_string()) {
        return Err(DrillError::NotBlacklisted {
            unit: unit.to_string(),
            file: file.to_string(),
        });
    }

    unit_config.blacklist.retain(|f| f != file);
    config.save()?;

    println!("Removed '{file}' from blacklist in unit '{unit}'");
    Ok(())
}

pub fn blacklist_list(config: &Config, unit_filter: Option<&str>) -> Result<(), DrillError> {
    let units: Vec<(&str, &UnitConfig)> = if let Some(name) = unit_filter {
        let unit = config
            .units
            .get(name)
            .ok_or_else(|| DrillError::UnitNotFound(name.to_string()))?;
        vec![(name, unit)]
    } else {
        config.units.iter().map(|(k, v)| (k.as_str(), v)).collect()
    };

    let mut any = false;
    for (name, unit) in &units {
        if !unit.blacklist.is_empty() {
            println!("  [{name}]");
            for file in &unit.blacklist {
                println!("    {file}");
            }
            any = true;
        }
    }
    if !any {
        println!("No blacklisted files.");
    }
    Ok(())
}

pub fn history(state: &State, days: u32) -> Result<(), DrillError> {
    let today = Local::now().date_naive();
    let cutoff = today - Days::new(u64::from(days));

    let mut entries: Vec<_> = state
        .sessions
        .iter()
        .filter(|(_, session)| session.date >= cutoff)
        .collect();
    entries.sort_by(|a, b| b.0.cmp(a.0));

    if entries.is_empty() {
        println!("No sessions in the last {days} days.");
        return Ok(());
    }

    for (_, session) in &entries {
        display::print_history_entry(session);
    }
    Ok(())
}

pub fn status(config: &Config, state: &State) -> Result<(), DrillError> {
    if config.units.is_empty() {
        println!("No units configured.");
        return Ok(());
    }

    for (name, unit) in &config.units {
        let total = scanner::scan_exercises(&unit.path, &unit.extension)
            .map(|f| f.len())
            .unwrap_or(0);
        let blacklisted = unit.blacklist.len();
        let available = total.saturating_sub(blacklisted);
        let cycle = state.cycles.get(name);
        let proposed = cycle.map_or(0, |c| c.proposed.len());
        let cycle_num = cycle.map_or(0, |c| c.cycle_number);
        let remaining = available.saturating_sub(proposed);

        let retry_count = state.retries.get(name).map_or(0, |r| r.len());

        display::print_status(&display::UnitStatus {
            name,
            total,
            blacklisted,
            available,
            proposed,
            remaining,
            cycle_number: cycle_num,
            retries: retry_count,
        });
    }
    Ok(())
}

pub fn more(
    config: &Config,
    state: &mut State,
    unit: &str,
    count: usize,
    skip_confirm: bool,
) -> Result<(), DrillError> {
    let unit_config = config
        .units
        .get(unit)
        .ok_or_else(|| DrillError::UnitNotFound(unit.to_string()))?;

    let today_key = Local::now().date_naive().to_string();
    let session = state
        .sessions
        .get(&today_key)
        .ok_or(DrillError::NoProposeToday)?;

    let already_today = session.selections.get(unit).map_or(0, |v| v.len());

    if !skip_confirm {
        print!("Already have {already_today} from '{unit}' today. Pick {count} more? [y/N] ");
        io::stdout().flush()?;
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        if !input.trim().eq_ignore_ascii_case("y") {
            println!("Cancelled.");
            return Ok(());
        }
    }

    let selected = proposer::select_more(unit, unit_config, state, count)?;

    // Append to today's session
    let session = state.sessions.get_mut(&today_key).unwrap();
    session
        .selections
        .entry(unit.to_string())
        .or_default()
        .extend(selected.clone());

    state.save()?;

    println!();
    println!("  [{unit}] extra");
    for file in &selected {
        println!("    {file}");
    }
    println!();

    Ok(())
}

pub fn fail(config: &Config, state: &mut State, unit: &str, file: &str) -> Result<(), DrillError> {
    let unit_config = config
        .units
        .get(unit)
        .ok_or_else(|| DrillError::UnitNotFound(unit.to_string()))?;

    let files = scanner::scan_exercises(&unit_config.path, &unit_config.extension)?;
    if !files.contains(&file.to_string()) {
        return Err(DrillError::FileNotInUnit {
            unit: unit.to_string(),
            file: file.to_string(),
        });
    }

    let retries = state.retries.entry(unit.to_string()).or_default();
    if retries.contains(&file.to_string()) {
        return Err(DrillError::AlreadyFailed {
            unit: unit.to_string(),
            file: file.to_string(),
        });
    }

    retries.push(file.to_string());
    retries.sort();
    state.save()?;

    println!("Marked '{file}' as failed in unit '{unit}' — will retry tomorrow");
    Ok(())
}

pub fn pass(config: &Config, state: &mut State, unit: &str, file: &str) -> Result<(), DrillError> {
    if !config.units.contains_key(unit) {
        return Err(DrillError::UnitNotFound(unit.to_string()));
    }

    let retries = state.retries.entry(unit.to_string()).or_default();
    if !retries.contains(&file.to_string()) {
        return Err(DrillError::NotFailed {
            unit: unit.to_string(),
            file: file.to_string(),
        });
    }

    retries.retain(|f| f != file);
    state.save()?;

    println!("Marked '{file}' as passed in unit '{unit}' — removed from retries");
    Ok(())
}

pub fn reset(
    config: &Config,
    state: &mut State,
    unit_filter: Option<&str>,
) -> Result<(), DrillError> {
    match unit_filter {
        Some(name) => {
            if !config.units.contains_key(name) {
                return Err(DrillError::UnitNotFound(name.to_string()));
            }
            state.cycles.remove(name);
            println!("Reset cycle for unit '{name}'");
        }
        None => {
            state.cycles.clear();
            println!("Reset all cycles");
        }
    }
    state.save()?;
    Ok(())
}
