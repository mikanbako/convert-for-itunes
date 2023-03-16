use std::{
    path::{Path, PathBuf},
    process::Command,
};

use tempfile::TempDir;
use which::which;

use crate::{conversion_error::ConversionError, utilities};

pub fn get_command(command: &str) -> Result<Command, ConversionError> {
    let command_path = which(command);

    match command_path {
        Ok(command_path) => Ok(Command::new(command_path)),

        Err(error) => Err(ConversionError::CommandNotFound {
            command: command.to_string(),
            error,
        }),
    }
}

pub fn run_command(command: &mut Command, command_name: &str) -> Result<(), ConversionError> {
    let result = command.status();

    match result {
        Ok(exit_status) => {
            if exit_status.success() {
                Ok(())
            } else {
                Err(ConversionError::CommandFailed {
                    command: command_name.to_string(),
                    status: exit_status,
                })
            }
        }
        Err(error) => Err(ConversionError::CommandCannotExecuted {
            command: command_name.to_string(),
            error,
        }),
    }
}

pub fn create_path_bufs(paths: &[&Path]) -> Vec<PathBuf> {
    paths.iter().map(|path| path.to_path_buf()).collect()
}

pub fn create_temporary_wav_file_path() -> Result<(PathBuf, TempDir), ConversionError> {
    let temporary_directory = utilities::create_temporary_directory()?;

    let wav_path = {
        let mut path = PathBuf::from(temporary_directory.path());

        path.push("source.wav");

        path
    };

    Ok((wav_path, temporary_directory))
}

pub fn has_extension<P: AsRef<Path>>(extension: &str, file: P) -> bool {
    match file.as_ref().extension() {
        Some(file_extension) => file_extension.eq_ignore_ascii_case(extension),
        None => false,
    }
}

pub fn has_all_extension<'a, F>(paths: &[&'a Path], check: F) -> bool
where
    F: Fn(&'a Path) -> bool,
{
    paths.iter().all(|path| check(path))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn check_ogg_extension_with_smallcase() {
        assert!(has_extension("ogg", "test.ogg"));
    }

    #[test]
    fn check_ogg_extension_with_uppercase() {
        assert!(has_extension("ogg", "test.OGG"));
    }

    #[test]
    fn check_no_ogg_extension() {
        assert!(!has_extension("ogg", "file.flac"));
    }
}
