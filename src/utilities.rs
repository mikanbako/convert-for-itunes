use std::path::{Path, PathBuf};

use tempfile::{tempdir, TempDir};

use crate::conversion_error::ConversionError;

pub fn get_paths_from_path_bufs(path_bufs: &[PathBuf]) -> Vec<&Path> {
    path_bufs.iter().map(PathBuf::as_path).collect()
}

pub fn create_temporary_directory() -> Result<TempDir, ConversionError> {
    match tempdir() {
        Ok(directory) => Ok(directory),
        Err(error) => Err(ConversionError::IoError { error }),
    }
}

pub fn filter_paths<T: AsRef<Path>>(paths: &[T]) -> Vec<PathBuf> {
    const EXCLUDED_EXTENSIONS: &[&str] = &["log", "txt"];

    paths
        .iter()
        .filter_map(|path| {
            let path = path.as_ref();

            if let Some(extension) = path.extension() {
                if let Some(extension) = extension.to_ascii_lowercase().to_str() {
                    if !EXCLUDED_EXTENSIONS.contains(&extension) {
                        return Some(path);
                    }
                }
            }

            None
        })
        .map(PathBuf::from)
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
