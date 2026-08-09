use crate::state::Session;

pub fn print_session(session: &Session, is_existing: bool) {
    if is_existing {
        println!("drill — {} (already proposed)", session.date);
    } else {
        println!("drill — {}", session.date);
    }
    println!();
    for (unit_name, files) in &session.selections {
        println!("  [{unit_name}]");
        for file in files {
            println!("    {file}");
        }
        println!();
    }
}

pub fn print_status(
    name: &str,
    total: usize,
    blacklisted: usize,
    available: usize,
    proposed: usize,
    remaining: usize,
    cycle_number: u32,
) {
    println!("  [{name}]");
    println!("    total:       {total}");
    println!("    blacklisted: {blacklisted}");
    println!("    available:   {available}");
    println!("    proposed:    {proposed} (cycle #{cycle_number})");
    println!("    remaining:   {remaining}");
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
