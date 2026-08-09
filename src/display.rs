use std::collections::HashMap;

use crate::state::Session;

pub fn print_session(
    session: &Session,
    retries: &HashMap<String, Vec<String>>,
    is_existing: bool,
) {
    if is_existing {
        println!("drill — {} (already proposed)", session.date);
    } else {
        println!("drill — {}", session.date);
    }
    println!();

    // Show retries first
    for (unit_name, files) in retries {
        if !files.is_empty() {
            println!("  [{unit_name}] retry");
            for file in files {
                println!("    {file}");
            }
            println!();
        }
    }

    // Then fresh picks
    for (unit_name, files) in &session.selections {
        println!("  [{unit_name}]");
        for file in files {
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
    println!("    proposed:    {} (cycle #{})", s.proposed, s.cycle_number);
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
