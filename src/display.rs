use std::collections::HashMap;

use crate::{config::Config, state::Session};

fn unit_sort_key(name: &str, config: &Config) -> (u32, String) {
    let order = config
        .units
        .get(name)
        .and_then(|u| u.order)
        .unwrap_or(u32::MAX);
    (order, name.to_string())
}

pub fn print_session(
    session: &Session,
    retries: &HashMap<String, Vec<String>>,
    config: &Config,
    is_existing: bool,
) {
    if is_existing {
        println!("drill — {} (already proposed)", session.date);
    } else {
        println!("drill — {}", session.date);
    }
    println!();

    // Show retries first, sorted by unit order
    let mut retry_units: Vec<_> = retries
        .iter()
        .filter(|(_, files)| !files.is_empty())
        .collect();
    retry_units.sort_by(|(a, _), (b, _)| unit_sort_key(a, config).cmp(&unit_sort_key(b, config)));
    for (unit_name, files) in &retry_units {
        println!("  [{unit_name}] retry");
        for file in *files {
            println!("    {file}");
        }
        println!();
    }

    // Then fresh picks, sorted by unit order
    let mut selection_units: Vec<_> = session.selections.iter().collect();
    selection_units
        .sort_by(|(a, _), (b, _)| unit_sort_key(a, config).cmp(&unit_sort_key(b, config)));
    for (unit_name, files) in &selection_units {
        println!("  [{unit_name}]");
        for file in *files {
            println!("    {file}");
        }
        println!();
    }
}

pub struct UnitStatus<'a> {
    pub name: &'a str,
    pub total: usize,
    pub blacklisted: usize,
    pub available: usize,
    pub proposed: usize,
    pub remaining: usize,
    pub cycle_number: u32,
    pub retries: usize,
}

pub fn print_status(s: &UnitStatus<'_>) {
    println!("  [{}]", s.name);
    println!("    total:       {}", s.total);
    println!("    blacklisted: {}", s.blacklisted);
    println!("    available:   {}", s.available);
    println!(
        "    proposed:    {} (cycle #{})",
        s.proposed, s.cycle_number
    );
    println!("    remaining:   {}", s.remaining);
    if s.retries > 0 {
        println!("    retries:     {}", s.retries);
    }
    println!();
}

pub fn print_history_entry(session: &Session) {
    println!("{}:", session.date);
    for (unit, files) in &session.selections {
        println!("  [{unit}]");
        for file in files {
            println!("    {file}");
        }
    }
    println!();
}
