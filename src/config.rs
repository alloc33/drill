use std::{collections::BTreeMap, fs, path::PathBuf};

use serde::{Deserialize, Serialize};

use crate::error::DrillError;

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub units: BTreeMap<String, UnitConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnitConfig {
    pub path: PathBuf,
    pub extension: String,
    pub count: usize,
    #[serde(default)]
    pub blacklist: Vec<String>,
    #[serde(default)]
    pub order: Option<u32>,
}

impl Config {
    /// Units sorted by order (then alphabetically).
    pub fn units_ordered(&self) -> Vec<(&String, &UnitConfig)> {
        let mut units: Vec<_> = self.units.iter().collect();
        units.sort_by(|(a_name, a), (b_name, b)| {
            a.order
                .unwrap_or(u32::MAX)
                .cmp(&b.order.unwrap_or(u32::MAX))
                .then_with(|| a_name.cmp(b_name))
        });
        units
    }

    fn config_path() -> Result<PathBuf, DrillError> {
        let dir = dirs::config_dir()
            .ok_or_else(|| {
                DrillError::Io(std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    "could not determine config directory",
                ))
            })?
            .join("drill");
        Ok(dir.join("config.toml"))
    }

    pub fn load() -> Result<Self, DrillError> {
        let path = Self::config_path()?;
        if !path.exists() {
            return Ok(Config::default());
        }
        let contents = fs::read_to_string(&path)?;
        Ok(toml::from_str(&contents)?)
    }

    pub fn save(&self) -> Result<(), DrillError> {
        let path = Self::config_path()?;
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let contents = toml::to_string_pretty(self)?;
        fs::write(&path, contents)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serde_round_trip() {
        let mut config = Config::default();
        config.units.insert(
            "dsa".to_string(),
            UnitConfig {
                path: PathBuf::from("/some/path"),
                extension: "ers".to_string(),
                count: 4,
                blacklist: vec!["027_lru_cache.ers".to_string()],
                order: None,
            },
        );

        let serialized = toml::to_string_pretty(&config).unwrap();
        let deserialized: Config = toml::from_str(&serialized).unwrap();
        let reserialized = toml::to_string_pretty(&deserialized).unwrap();

        assert_eq!(serialized, reserialized);
    }

    #[test]
    fn empty_config_round_trip() {
        let config = Config::default();
        let serialized = toml::to_string_pretty(&config).unwrap();
        let deserialized: Config = toml::from_str(&serialized).unwrap();
        assert!(deserialized.units.is_empty());
    }
}
