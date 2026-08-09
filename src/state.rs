use std::{
    collections::{BTreeMap, HashMap},
    fs,
    path::PathBuf,
};

use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

use crate::error::DrillError;

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct State {
    #[serde(default)]
    pub cycles: HashMap<String, CycleState>,
    #[serde(default)]
    pub sessions: BTreeMap<String, Session>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CycleState {
    pub proposed: Vec<String>,
    pub cycle_number: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub date: NaiveDate,
    pub selections: BTreeMap<String, Vec<String>>,
}

impl State {
    fn state_path() -> Result<PathBuf, DrillError> {
        let dir = dirs::data_dir()
            .ok_or_else(|| {
                DrillError::Io(std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    "could not determine data directory",
                ))
            })?
            .join("drill");
        Ok(dir.join("state.json"))
    }

    pub fn load() -> Result<Self, DrillError> {
        let path = Self::state_path()?;
        if !path.exists() {
            return Ok(State::default());
        }
        let contents = fs::read_to_string(&path)?;
        Ok(serde_json::from_str(&contents)?)
    }

    pub fn save(&self) -> Result<(), DrillError> {
        let path = Self::state_path()?;
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let contents = serde_json::to_string_pretty(self)?;
        fs::write(&path, contents)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serde_round_trip() {
        let mut state = State::default();
        state.cycles.insert(
            "dsa".to_string(),
            CycleState {
                proposed: vec!["001_a.ers".to_string(), "002_b.ers".to_string()],
                cycle_number: 3,
            },
        );

        let date = NaiveDate::from_ymd_opt(2026, 8, 9).unwrap();
        let mut selections = BTreeMap::new();
        selections.insert(
            "dsa".to_string(),
            vec!["001_a.ers".to_string(), "002_b.ers".to_string()],
        );
        state
            .sessions
            .insert(date.to_string(), Session { date, selections });

        let serialized = serde_json::to_string_pretty(&state).unwrap();
        let deserialized: State = serde_json::from_str(&serialized).unwrap();
        let reserialized = serde_json::to_string_pretty(&deserialized).unwrap();

        assert_eq!(serialized, reserialized);
    }
}
