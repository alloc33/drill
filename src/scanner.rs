use std::{fs, path::Path};

use crate::error::DrillError;

/// Scan a directory for exercise files with the given extension.
/// Top-level only — does not recurse into subdirectories.
pub fn scan_exercises(path: &Path, extension: &str) -> Result<Vec<String>, DrillError> {
    if !path.exists() {
        return Err(DrillError::PathNotFound(path.to_path_buf()));
    }

    let suffix = format!(".{extension}");
    let mut files = Vec::new();

    for entry in fs::read_dir(path)? {
        let entry = entry?;
        if !entry.file_type()?.is_file() {
            continue;
        }
        let name = entry.file_name().to_string_lossy().into_owned();
        if name.ends_with(&suffix) {
            files.push(name);
        }
    }

    files.sort();
    Ok(files)
}

#[cfg(test)]
mod tests {
    use std::fs;

    use tempfile::TempDir;

    use super::*;

    fn create_files(dir: &TempDir, names: &[&str]) {
        for name in names {
            fs::write(dir.path().join(name), "").unwrap();
        }
    }

    #[test]
    fn finds_matching_files() {
        let dir = tempfile::tempdir().unwrap();
        create_files(&dir, &["001_a.ers", "002_b.ers", "003_c.ers"]);

        let result = scan_exercises(dir.path(), "ers").unwrap();
        assert_eq!(result, vec!["001_a.ers", "002_b.ers", "003_c.ers"]);
    }

    #[test]
    fn ignores_other_extensions() {
        let dir = tempfile::tempdir().unwrap();
        create_files(&dir, &["001_a.ers", "notes.txt", "readme.md", "002_b.ers"]);

        let result = scan_exercises(dir.path(), "ers").unwrap();
        assert_eq!(result, vec!["001_a.ers", "002_b.ers"]);
    }

    #[test]
    fn ignores_subdirectories() {
        let dir = tempfile::tempdir().unwrap();
        create_files(&dir, &["001_a.ers"]);
        fs::create_dir(dir.path().join("practice")).unwrap();
        fs::write(dir.path().join("practice/001_a.ers"), "").unwrap();

        let result = scan_exercises(dir.path(), "ers").unwrap();
        assert_eq!(result, vec!["001_a.ers"]);
    }

    #[test]
    fn returns_sorted() {
        let dir = tempfile::tempdir().unwrap();
        create_files(&dir, &["003_c.ers", "001_a.ers", "002_b.ers"]);

        let result = scan_exercises(dir.path(), "ers").unwrap();
        assert_eq!(result, vec!["001_a.ers", "002_b.ers", "003_c.ers"]);
    }

    #[test]
    fn error_on_missing_path() {
        let result = scan_exercises(Path::new("/nonexistent/path"), "ers");
        assert!(result.is_err());
    }

    #[test]
    fn empty_directory() {
        let dir = tempfile::tempdir().unwrap();
        let result = scan_exercises(dir.path(), "ers").unwrap();
        assert!(result.is_empty());
    }
}
