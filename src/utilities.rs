//! Utility functions.

use std::{
    ffi::OsStr,
    path::{Path, PathBuf},
};

use tempfile::{tempdir, TempDir};

use crate::conversion_error::ConversionError;

/// Converts a slice of [`PathBuf`] to a vector of &[`Path`].
pub fn get_paths_from_path_bufs(path_bufs: &[PathBuf]) -> Vec<&Path> {
    path_bufs.iter().map(PathBuf::as_path).collect()
}

/// Creates a temporary directory.
pub fn create_temporary_directory() -> Result<TempDir, ConversionError> {
    tempdir().map_err(|error| ConversionError::IoError { error })
}

/// Filters paths by extensions.
///
/// For example .log and .txt files are excluded.
pub fn filter_paths<T: AsRef<Path>>(paths: &[T]) -> Vec<PathBuf> {
    const EXCLUDED_EXTENSIONS: &[&str] = &["log", "txt"];

    fn is_included_extension(extension: &OsStr) -> bool {
        !EXCLUDED_EXTENSIONS
            .iter()
            .any(|excluded_extension| *excluded_extension == extension.to_ascii_lowercase())
    }

    paths
        .iter()
        .filter_map(|path| {
            let path = path.as_ref();

            path.extension()
                .filter(|extension| is_included_extension(extension))
                .map(|_| PathBuf::from(path))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_filter_paths() {
        let mp3_path = Path::new("a.mp3");
        let log_path = Path::new("b.log");
        let txt_path = Path::new("c.txt");

        let result = filter_paths(&[mp3_path, log_path, txt_path]);

        assert_eq!(1, result.len());
        assert!(result.contains(&mp3_path.to_path_buf()));
    }

    #[test]
    fn test_filter_paths_with_uppercase() {
        let mp3_path = Path::new("a.mp3");
        let log_path = Path::new("b.LOG");
        let txt_path = Path::new("c.TXT");

        let result = filter_paths(&[mp3_path, log_path, txt_path]);

        assert_eq!(1, result.len());
        assert!(result.contains(&mp3_path.to_path_buf()));
    }
}
