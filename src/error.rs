use std::path::PathBuf;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum DrillError {
    #[error("unit '{0}' not found")]
    UnitNotFound(String),

    #[error("unit '{0}' already exists")]
    UnitAlreadyExists(String),

    #[error("path does not exist: {}", .0.display())]
    PathNotFound(PathBuf),

    #[error("no exercises found in {} with extension .{ext}", .path.display())]
    NoExercisesFound { path: PathBuf, ext: String },

    #[error("file '{file}' not found in unit '{unit}'")]
    FileNotInUnit { unit: String, file: String },

    #[error("file '{file}' already blacklisted in unit '{unit}'")]
    AlreadyBlacklisted { unit: String, file: String },

    #[error("file '{file}' not blacklisted in unit '{unit}'")]
    NotBlacklisted { unit: String, file: String },

    #[error("file '{file}' already marked as failed in unit '{unit}'")]
    AlreadyFailed { unit: String, file: String },

    #[error("file '{file}' not in retry list for unit '{unit}'")]
    NotFailed { unit: String, file: String },

    #[error("{0}")]
    Io(#[from] std::io::Error),

    #[error("{0}")]
    TomlSer(#[from] toml::ser::Error),

    #[error("{0}")]
    TomlDe(#[from] toml::de::Error),

    #[error("{0}")]
    Json(#[from] serde_json::Error),
}
