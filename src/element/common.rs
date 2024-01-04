//! Common functions for elements.

use std::{
    path::{Path, PathBuf},
    process::Command,
};

use tempfile::TempDir;
use which::which;

use crate::{conversion_error::ConversionError, utilities};

/// Gets a [`Command`] from a string that represents a command.
pub fn get_command(command: &str) -> Result<Command, ConversionError> {
    let command_path = which(command);

    command_path
        .map(Command::new)
        .map_err(|error| ConversionError::CommandNotFound {
            command: command.to_string(),
            error,
        })
}

/// Runs a command.
pub fn run_command(command: &mut Command) -> Result<(), ConversionError> {
    let result = command.status();

    fn get_program(command: &Command) -> String {
        command.get_program().to_string_lossy().to_string()
    }

    let exit_status = result.map_err(|error| ConversionError::CommandCannotExecuted {
        command: get_program(command),
        error,
    })?;

    if exit_status.success() {
        Ok(())
    } else {
        Err(ConversionError::CommandFailed {
            command: get_program(command),
            status: exit_status,
        })
    }
}

/// Creates a [`Vec`] of [`PathBuf`] from a slice of &[`Path`].
pub fn create_path_bufs(paths: &[&Path]) -> Vec<PathBuf> {
    paths.iter().map(|path| path.to_path_buf()).collect()
}

/// Creates a temporary file for wav file to decode compressed music file.
///
/// This function returns the path of a temporary file and the directory that contains it
/// to keep the directory.
pub fn create_temporary_wav_file_path() -> Result<(PathBuf, TempDir), ConversionError> {
    let temporary_directory = utilities::create_temporary_directory()?;

    let wav_path = {
        let mut path = PathBuf::from(temporary_directory.path());

        path.push("source.wav");

        path
    };

    Ok((wav_path, temporary_directory))
}

/// Whether a `file` has the `extension`.
pub fn has_extension<P: AsRef<Path>>(extension: &str, file: P) -> bool {
    file.as_ref().extension().map_or(false, |file_extension| {
        file_extension.eq_ignore_ascii_case(extension)
    })
}

/// Whether all files have the extension that checks by `check` function.
pub fn has_all_extension<'a, F>(files: &[&'a Path], check: F) -> bool
where
    F: Fn(&'a Path) -> bool,
{
    files.iter().all(|path| check(path))
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
