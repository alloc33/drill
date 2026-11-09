use std::collections::BTreeMap;

use chrono::{Local, NaiveDate};
use rand::seq::SliceRandom;

use crate::{
    config::Config,
    error::DrillError,
    scanner,
    state::{Session, State},
};

/// Returns `(session, is_existing)`.
/// If today's session was already generated, returns it unchanged.
pub fn propose(config: &Config, state: &mut State) -> Result<(Session, bool), DrillError> {
    let today = Local::now().date_naive();
    let result = generate_session(config, state, today)?;
    state.save()?;
    Ok(result)
}

fn generate_session(
    config: &Config,
    state: &mut State,
    date: NaiveDate,
) -> Result<(Session, bool), DrillError> {
    let date_key = date.to_string();

    if let Some(existing) = state.sessions.get(&date_key) {
        return Ok((existing.clone(), true));
    }

    let mut selections = BTreeMap::new();

    for (unit_name, unit_config) in config.units_ordered() {
        let selected = select_for_unit(unit_name, unit_config, state)?;
        selections.insert(unit_name.clone(), selected);
    }

    let session = Session { date, selections };

    state.sessions.insert(date_key, session.clone());

    Ok((session, false))
}

fn select_for_unit(
    unit_name: &str,
    unit_config: &crate::config::UnitConfig,
    state: &mut State,
) -> Result<Vec<String>, DrillError> {
    select_n_from_unit(unit_name, unit_config, state, unit_config.count)
}

/// Pick extra exercises from a unit. Same cycle tracking as propose.
pub fn select_more(
    unit_name: &str,
    unit_config: &crate::config::UnitConfig,
    state: &mut State,
    count: usize,
) -> Result<Vec<String>, DrillError> {
    select_n_from_unit(unit_name, unit_config, state, count)
}

fn select_n_from_unit(
    unit_name: &str,
    unit_config: &crate::config::UnitConfig,
    state: &mut State,
    count: usize,
) -> Result<Vec<String>, DrillError> {
    let all_files = scanner::scan_exercises(&unit_config.path, &unit_config.extension)?;

    let available: Vec<String> = all_files
        .into_iter()
        .filter(|f| !unit_config.blacklist.contains(f))
        .collect();

    if available.is_empty() {
        eprintln!("warning: unit '{unit_name}': no available exercises");
        return Ok(Vec::new());
    }

    let cycle = state.cycles.entry(unit_name.to_string()).or_default();

    // Prune proposed entries for files that no longer exist on disk
    cycle.proposed.retain(|f| available.contains(f));

    let remaining: Vec<String> = available
        .iter()
        .filter(|f| !cycle.proposed.contains(f))
        .cloned()
        .collect();

    let selected = if remaining.len() >= count {
        random_sample(&remaining, count)
    } else {
        // Cycle boundary: take all remaining, reset, pick deficit from fresh pool
        let mut picks = remaining.clone();
        let deficit = count - picks.len();

        cycle.proposed.clear();
        cycle.cycle_number += 1;

        // Exclude yesterday's picks to avoid back-to-back repeats at cycle boundary
        let last_session_picks: Vec<&String> = state
            .sessions
            .values()
            .next_back()
            .and_then(|s| s.selections.get(unit_name))
            .map_or(Vec::new(), |v| v.iter().collect());

        let fresh_pool: Vec<String> = available
            .iter()
            .filter(|f| !picks.contains(f) && !last_session_picks.contains(f))
            .cloned()
            .collect();

        if deficit >= fresh_pool.len() {
            picks.extend(fresh_pool);
        } else {
            picks.extend(random_sample(&fresh_pool, deficit));
        }

        picks
    };

    cycle.proposed.extend(selected.clone());

    if selected.len() < count {
        eprintln!(
            "warning: unit '{unit_name}': only {} exercises available (requested {count})",
            selected.len(),
        );
    }

    Ok(selected)
}

fn random_sample(pool: &[String], count: usize) -> Vec<String> {
    let mut rng = rand::rng();
    let mut pool = pool.to_vec();
    pool.shuffle(&mut rng);
    pool.into_iter().take(count).collect()
}

#[cfg(test)]
mod tests {
    use std::{collections::HashSet, fs, path::PathBuf};

    use tempfile::TempDir;

    use super::*;
    use crate::config::UnitConfig;

    fn create_exercises(dir: &TempDir, names: &[&str]) {
        for name in names {
            fs::write(dir.path().join(name), "").unwrap();
        }
    }

    fn make_unit(dir: &TempDir, count: usize) -> UnitConfig {
        UnitConfig {
            path: dir.path().to_path_buf(),
            extension: "ers".to_string(),
            count,
            blacklist: Vec::new(),
            order: None,
        }
    }

    fn exercise_names(n: usize) -> Vec<String> {
        (1..=n).map(|i| format!("{i:03}_exercise.ers")).collect()
    }

    #[test]
    fn selects_correct_count() {
        let dir = tempfile::tempdir().unwrap();
        let names: Vec<&str> = vec![
            "001_a.ers",
            "002_b.ers",
            "003_c.ers",
            "004_d.ers",
            "005_e.ers",
        ];
        create_exercises(&dir, &names);
        let unit = make_unit(&dir, 3);
        let mut state = State::default();

        let selected = select_for_unit("test", &unit, &mut state).unwrap();
        assert_eq!(selected.len(), 3);
        for s in &selected {
            assert!(names.contains(&s.as_str()));
        }
    }

    #[test]
    fn excludes_blacklisted() {
        let dir = tempfile::tempdir().unwrap();
        create_exercises(&dir, &["001_a.ers", "002_b.ers", "003_c.ers"]);
        let mut unit = make_unit(&dir, 2);
        unit.blacklist = vec!["002_b.ers".to_string()];
        let mut state = State::default();

        for _ in 0..20 {
            state.cycles.clear();
            let selected = select_for_unit("test", &unit, &mut state).unwrap();
            assert!(!selected.contains(&"002_b.ers".to_string()));
        }
    }

    #[test]
    fn no_repeats_within_cycle() {
        let dir = tempfile::tempdir().unwrap();
        let names = exercise_names(10);
        let name_refs: Vec<&str> = names.iter().map(|s| s.as_str()).collect();
        create_exercises(&dir, &name_refs);
        let unit = make_unit(&dir, 3);
        let mut state = State::default();

        let first = select_for_unit("test", &unit, &mut state).unwrap();
        let second = select_for_unit("test", &unit, &mut state).unwrap();

        let first_set: HashSet<_> = first.into_iter().collect();
        let second_set: HashSet<_> = second.into_iter().collect();
        assert!(first_set.is_disjoint(&second_set));
    }

    #[test]
    fn cycle_boundary_crossing() {
        let dir = tempfile::tempdir().unwrap();
        let names = exercise_names(5);
        let name_refs: Vec<&str> = names.iter().map(|s| s.as_str()).collect();
        create_exercises(&dir, &name_refs);
        let unit = make_unit(&dir, 3);
        let mut state = State::default();

        // Pick 3 of 5 — remaining = 2
        let first = select_for_unit("test", &unit, &mut state).unwrap();
        assert_eq!(first.len(), 3);
        assert_eq!(state.cycles["test"].cycle_number, 0);

        // Pick 3 more — only 2 left, crosses cycle boundary
        let second = select_for_unit("test", &unit, &mut state).unwrap();
        assert_eq!(second.len(), 3);
        assert_eq!(state.cycles["test"].cycle_number, 1);

        // The 2 carried over from old cycle should not repeat in the deficit picks
        let second_set: HashSet<_> = second.iter().collect();
        assert_eq!(second_set.len(), 3); // no duplicates within the pick
    }

    #[test]
    fn full_cycle_coverage() {
        let dir = tempfile::tempdir().unwrap();
        let names = exercise_names(8);
        let name_refs: Vec<&str> = names.iter().map(|s| s.as_str()).collect();
        create_exercises(&dir, &name_refs);
        let unit = make_unit(&dir, 2);
        let mut state = State::default();

        let mut seen: HashSet<String> = HashSet::new();
        // 8 exercises, 2 per pick = 4 picks to see all
        for _ in 0..4 {
            let selected = select_for_unit("test", &unit, &mut state).unwrap();
            assert_eq!(selected.len(), 2);
            for s in selected {
                assert!(seen.insert(s), "duplicate before full cycle");
            }
        }
        let all: HashSet<_> = names.into_iter().collect();
        assert_eq!(seen, all);
    }

    #[test]
    fn count_exceeds_available() {
        let dir = tempfile::tempdir().unwrap();
        create_exercises(&dir, &["001_a.ers", "002_b.ers"]);
        let unit = make_unit(&dir, 5);
        let mut state = State::default();

        let selected = select_for_unit("test", &unit, &mut state).unwrap();
        assert_eq!(selected.len(), 2);
    }

    #[test]
    fn all_blacklisted_returns_empty() {
        let dir = tempfile::tempdir().unwrap();
        create_exercises(&dir, &["001_a.ers", "002_b.ers"]);
        let mut unit = make_unit(&dir, 2);
        unit.blacklist = vec!["001_a.ers".to_string(), "002_b.ers".to_string()];
        let mut state = State::default();

        let selected = select_for_unit("test", &unit, &mut state).unwrap();
        assert!(selected.is_empty());
    }

    #[test]
    fn prunes_stale_proposed() {
        let dir = tempfile::tempdir().unwrap();
        create_exercises(&dir, &["001_a.ers", "002_b.ers", "003_c.ers"]);
        let unit = make_unit(&dir, 2);
        let mut state = State::default();

        // Pretend "999_gone.ers" was proposed before but file was deleted
        state.cycles.insert(
            "test".to_string(),
            crate::state::CycleState {
                proposed: vec!["999_gone.ers".to_string(), "001_a.ers".to_string()],
                cycle_number: 0,
            },
        );

        let selected = select_for_unit("test", &unit, &mut state).unwrap();
        assert_eq!(selected.len(), 2);
        assert!(!selected.contains(&"001_a.ers".to_string()));
        // Stale entry should be pruned
        assert!(
            !state.cycles["test"]
                .proposed
                .contains(&"999_gone.ers".to_string())
        );
    }

    #[test]
    fn daily_determinism() {
        let dir = tempfile::tempdir().unwrap();
        let names = exercise_names(10);
        let name_refs: Vec<&str> = names.iter().map(|s| s.as_str()).collect();
        create_exercises(&dir, &name_refs);

        let mut config = Config::default();
        config.units.insert(
            "test".to_string(),
            UnitConfig {
                path: dir.path().to_path_buf(),
                extension: "ers".to_string(),
                count: 3,
                blacklist: Vec::new(),
                order: None,
            },
        );

        let date = NaiveDate::from_ymd_opt(2026, 1, 15).unwrap();
        let mut state = State::default();

        let (first, is_new) = generate_session(&config, &mut state, date).unwrap();
        assert!(!is_new);

        let (second, is_existing) = generate_session(&config, &mut state, date).unwrap();
        assert!(is_existing);
        assert_eq!(first.selections, second.selections);
    }

    #[test]
    fn different_dates_give_independent_sessions() {
        let dir = tempfile::tempdir().unwrap();
        let names = exercise_names(10);
        let name_refs: Vec<&str> = names.iter().map(|s| s.as_str()).collect();
        create_exercises(&dir, &name_refs);

        let mut config = Config::default();
        config.units.insert(
            "test".to_string(),
            UnitConfig {
                path: PathBuf::from(dir.path()),
                extension: "ers".to_string(),
                count: 3,
                blacklist: Vec::new(),
                order: None,
            },
        );

        let mut state = State::default();
        let day1 = NaiveDate::from_ymd_opt(2026, 1, 1).unwrap();
        let day2 = NaiveDate::from_ymd_opt(2026, 1, 2).unwrap();

        let (s1, _) = generate_session(&config, &mut state, day1).unwrap();
        let (s2, _) = generate_session(&config, &mut state, day2).unwrap();

        assert_eq!(state.sessions.len(), 2);
        assert_eq!(s1.date, day1);
        assert_eq!(s2.date, day2);
    }
}
