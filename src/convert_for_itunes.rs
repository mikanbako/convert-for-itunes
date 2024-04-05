// SPDX-FileCopyrightText: 2024 Keita Kita <maoutwo@gmail.com>
//
// SPDX-License-Identifier: MIT

//! This module has the function that called by the main function.

use std::{
    fs::create_dir_all,
    io,
    path::{Path, PathBuf},
};

use clap::Parser;
use log::{debug, info};
use thiserror::Error;

use crate::{
    conversion_error::ConversionError,
    file_mover::{self, FileMovingError, MovedFile},
    music_converter::{self, ConvertedFile},
    utilities,
};

/// The struct for setting.
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = "Convert music files for iTunes.")]
pub struct Setting {
    #[arg(
        short,
        long,
        value_name = "DIRECTORY",
        help = "A destination directory that source files are moved."
    )]
    move_source_file_to: Option<PathBuf>,

    #[arg(
        required = true,
        value_parser = is_destination_directory_or_not_found,
        help = "A destination directory that saving MP3 files."
    )]
    destination_directory: PathBuf,

    #[arg(
        required = true,
        value_name = "SOURCE_FILE",
        value_parser = is_source_file_available,
        help = "MP3, Flac, AAC, WAV or Ogg Vorbis files in an album."
    )]
    source_files: Vec<PathBuf>,
}

/// Result of convert_for_itunes.
#[derive(Debug)]
pub struct OutputResult {
    /// The path of the source music file.
    pub source: PathBuf,

    /// The path of the converted file.
    pub converted_destination: PathBuf,

    /// The path of the moving source file.
    pub moving_destination: Option<PathBuf>,
}

/// Error of convert_for_itunes.
#[derive(Error, Debug)]
pub enum ConvertForITunesError {
    #[error("The directory `{0}` cannot be created: {1}")]
    DirectoryCannotBeCreated(PathBuf, io::Error),

    #[error("Conversion is failed: {0}")]
    ConversionError(ConversionError),

    #[error("Moving source file is failed: {0}")]
    MovingSourceFileIsFailed(FileMovingError),
}

#[cfg_attr(test, mockall::automock)]
#[allow(clippy::needless_lifetimes)]
trait ConvertForItunesRunner {
    fn create_destination_directory(&self, path: &Path) -> std::io::Result<()>;

    fn convert_all<'a>(
        &self,
        source_files_in_album: &[&'a Path],
        destination_directory: &Path,
    ) -> Result<Vec<ConvertedFile>, ConversionError>;

    fn move_files<'a>(
        &self,
        moving_files: &[&'a Path],
        destination_directory: &Path,
    ) -> Result<Vec<MovedFile>, FileMovingError>;
}

struct ConvertForItunes;

impl ConvertForItunesRunner for ConvertForItunes {
    fn create_destination_directory(&self, path: &Path) -> std::io::Result<()> {
        create_dir_all(path)
    }

    fn convert_all(
        &self,
        source_files_in_album: &[&Path],
        destination_directory: &Path,
    ) -> Result<Vec<ConvertedFile>, ConversionError> {
        music_converter::convert_all(source_files_in_album, destination_directory)
    }

    fn move_files(
        &self,
        moving_files: &[&Path],
        destination_directory: &Path,
    ) -> Result<Vec<MovedFile>, FileMovingError> {
        let file_mover = file_mover::FileMover::new(destination_directory)?;

        file_mover.move_files(moving_files)
    }
}

fn is_destination_directory_or_not_found(argument: &str) -> Result<PathBuf, String> {
    let path = Path::new(argument);

    if path.is_dir() || !path.exists() {
        Ok(path.to_path_buf())
    } else {
        Err(format!(
            r#"The destination "{argument}" exists and is not a directory."#
        ))
    }
}

fn is_source_file_available(argument: &str) -> Result<PathBuf, String> {
    let path = Path::new(argument);

    if path.is_file() {
        Ok(path.to_path_buf())
    } else {
        Err(format!(r#"The file "{argument}" is not found."#))
    }
}

fn create_output_results(
    converted_files: &[ConvertedFile],
    moved_files: &Option<&[MovedFile]>,
) -> Vec<OutputResult> {
    debug_assert!(moved_files.is_none() || (moved_files.unwrap().len() == converted_files.len()));

    fn get_moving_destination(
        source: &PathBuf,
        moved_files: &Option<&[MovedFile]>,
    ) -> Option<PathBuf> {
        moved_files.and_then(|moved_files| {
            moved_files
                .iter()
                .find(|moved_file| moved_file.source == *source)
                .map(|moved_file| moved_file.destination.clone())
        })
    }

    converted_files
        .iter()
        .map(|converted_file| OutputResult {
            source: converted_file.source.clone(),
            converted_destination: converted_file.destination.clone(),
            moving_destination: get_moving_destination(&converted_file.source, moved_files),
        })
        .collect()
}

fn log_about_starting(setting: &Setting) {
    if setting.move_source_file_to.is_some() {
        info!("Converts and moves music files.");
    } else {
        info!("Converts music files.");
    }
}

fn convert_for_itunes_on_runner<T: ConvertForItunesRunner>(
    setting: &Setting,
    runner: T,
) -> Result<Vec<OutputResult>, ConvertForITunesError> {
    log_about_starting(setting);

    debug!(
        "Destination directory for converted files: {:?}",
        &setting.destination_directory
    );

    runner
        .create_destination_directory(&setting.destination_directory)
        .map_err(|error| {
            ConvertForITunesError::DirectoryCannotBeCreated(
                setting.destination_directory.clone(),
                error,
            )
        })?;

    setting.move_source_file_to.as_ref().map_or(
        Ok(None),
        |moving_source_destination_directory| {
            debug!(
                "Destination directory for moved source files: {:?}",
                moving_source_destination_directory
            );

            runner
                .create_destination_directory(moving_source_destination_directory)
                .map(Some)
                .map_err(|error| {
                    ConvertForITunesError::DirectoryCannotBeCreated(
                        moving_source_destination_directory.clone(),
                        error,
                    )
                })
        },
    )?;

    let filtered_files = utilities::filter_paths(setting.source_files.as_slice());
    let source_paths = utilities::get_paths_from_path_bufs(&filtered_files);

    let converted_result = runner
        .convert_all(&source_paths, &setting.destination_directory)
        .map_err(ConvertForITunesError::ConversionError)?;

    let source_moved_result =
        setting
            .move_source_file_to
            .as_ref()
            .map_or(Ok(None), |moving_destination_directory| {
                info!("Moves source files.");

                runner
                    .move_files(&source_paths, moving_destination_directory)
                    .map(Some)
                    .map_err(ConvertForITunesError::MovingSourceFileIsFailed)
            })?;

    info!("Completed.");

    Ok(create_output_results(
        &converted_result,
        &source_moved_result.as_deref(),
    ))
}

/// Converts music files for iTunes.
///
/// Source music files are converted to MP3 files for iTunes and moved to the destination
/// directory. And the source music files may be moved to the another destination directory.
pub fn convert_for_itunes(setting: &Setting) -> Result<Vec<OutputResult>, ConvertForITunesError> {
    convert_for_itunes_on_runner(setting, ConvertForItunes)
}

#[cfg(test)]
mod tests {
    use std::ffi::OsStr;

    use mockall::predicate;
    use tempfile::{tempdir, NamedTempFile};

    use crate::music_converter::ConvertedFile;

    use super::*;

    fn get_output_result<'a, P: AsRef<Path>>(
        source: &P,
        output_results: &'a [OutputResult],
    ) -> Option<&'a OutputResult> {
        output_results
            .iter()
            .find(|result| result.source == source.as_ref())
    }

    #[test]
    fn convert_without_moved_files() {
        let source_file1 = PathBuf::from("source1.mp3");
        let source_file2 = PathBuf::from("source2.mp3");
        let destination_directory = PathBuf::from("destination");

        let setting = Setting {
            move_source_file_to: None,
            destination_directory: destination_directory.clone(),
            source_files: vec![source_file1.clone(), source_file2.clone()],
        };

        let converted_file1 = PathBuf::from("converted1");
        let converted_file2 = PathBuf::from("converted2");

        let runner = {
            let mut runner = MockConvertForItunesRunner::new();

            runner
                .expect_create_destination_directory()
                .with(predicate::eq(destination_directory))
                .times(1)
                .returning(|_| Ok(()));
            runner
                .expect_convert_all()
                .withf({
                    let expected_source_files = setting.source_files.clone();
                    let expected_destination_directory = setting.destination_directory.clone();
                    move |source_files, destination_directory| {
                        source_files == expected_source_files
                            && destination_directory == expected_destination_directory
                    }
                })
                .returning({
                    let source_file1 = source_file1.clone();
                    let source_file2 = source_file2.clone();
                    let converted_file1 = converted_file1.clone();
                    let converted_file2 = converted_file2.clone();
                    move |_, _| {
                        Ok(vec![
                            ConvertedFile {
                                source: source_file1.clone(),
                                destination: converted_file1.clone(),
                            },
                            ConvertedFile {
                                source: source_file2.clone(),
                                destination: converted_file2.clone(),
                            },
                        ])
                    }
                });

            runner
        };

        let output_results = convert_for_itunes_on_runner(&setting, runner).unwrap();

        assert_eq!(2, output_results.len());

        fn assert_output_result(converted_file: &ConvertedFile, output_results: &[OutputResult]) {
            let source_result = get_output_result(&converted_file.source, output_results).unwrap();

            assert_eq!(
                converted_file.destination,
                source_result.converted_destination
            );
            assert!(source_result.moving_destination.is_none());
        }

        assert_output_result(
            &ConvertedFile {
                source: source_file1,
                destination: converted_file1,
            },
            &output_results,
        );
        assert_output_result(
            &ConvertedFile {
                source: source_file2,
                destination: converted_file2,
            },
            &output_results,
        );
    }

    #[test]
    fn convert_with_moved_files() {
        let source_file1 = PathBuf::from("source1.mp3");
        let source_file2 = PathBuf::from("source2.mp3");
        let destination_directory = PathBuf::from("destination");
        let source_destination_directory = PathBuf::from("source_destination");

        let setting = Setting {
            move_source_file_to: Some(source_destination_directory.clone()),
            destination_directory: destination_directory.clone(),
            source_files: vec![source_file1.clone(), source_file2.clone()],
        };

        let converted_file1 = PathBuf::from("converted1");
        let converted_file2 = PathBuf::from("converted2");

        let moved_file1 = PathBuf::from("moved1");
        let moved_file2 = PathBuf::from("moved2");

        let runner = {
            let mut runner = MockConvertForItunesRunner::new();

            runner
                .expect_create_destination_directory()
                .with(predicate::eq(destination_directory))
                .times(1)
                .returning(|_| Ok(()));
            runner
                .expect_create_destination_directory()
                .with(predicate::eq(source_destination_directory))
                .times(1)
                .returning(|_| Ok(()));
            runner
                .expect_convert_all()
                .withf({
                    let expected_source_files = setting.source_files.clone();
                    let expected_destination_directory = setting.destination_directory.clone();
                    move |source_files, destination_directory| {
                        source_files == expected_source_files
                            && destination_directory == expected_destination_directory
                    }
                })
                .returning({
                    let source_file1 = source_file1.clone();
                    let source_file2 = source_file2.clone();
                    let converted_file1 = converted_file1.clone();
                    let converted_file2 = converted_file2.clone();
                    move |_, _| {
                        Ok(vec![
                            ConvertedFile {
                                source: source_file1.clone(),
                                destination: converted_file1.clone(),
                            },
                            ConvertedFile {
                                source: source_file2.clone(),
                                destination: converted_file2.clone(),
                            },
                        ])
                    }
                });
            runner
                .expect_move_files()
                .withf({
                    let expected_moving_files = setting.source_files.clone();
                    let expected_destination_directory =
                        setting.move_source_file_to.clone().unwrap();
                    move |moving_files, destination_directory| {
                        moving_files == expected_moving_files
                            && destination_directory == expected_destination_directory
                    }
                })
                .returning({
                    let source_file1 = source_file1.clone();
                    let source_file2 = source_file2.clone();
                    let moved_file1 = moved_file1.clone();
                    let moved_file2 = moved_file2.clone();
                    move |_, _| {
                        Ok(vec![
                            MovedFile {
                                source: source_file1.clone(),
                                destination: moved_file1.clone(),
                            },
                            MovedFile {
                                source: source_file2.clone(),
                                destination: moved_file2.clone(),
                            },
                        ])
                    }
                });

            runner
        };

        let output_results = convert_for_itunes_on_runner(&setting, runner).unwrap();

        assert_eq!(2, output_results.len());

        fn assert_output_result(
            converted_file: &ConvertedFile,
            moved_file: &PathBuf,
            output_results: &[OutputResult],
        ) {
            let source_result = get_output_result(&converted_file.source, output_results).unwrap();

            assert_eq!(
                converted_file.destination,
                source_result.converted_destination
            );
            assert_eq!(
                moved_file,
                source_result.moving_destination.as_ref().unwrap()
            );
        }

        assert_output_result(
            &ConvertedFile {
                source: source_file1,
                destination: converted_file1,
            },
            &moved_file1,
            &output_results,
        );
        assert_output_result(
            &ConvertedFile {
                source: source_file2,
                destination: converted_file2,
            },
            &moved_file2,
            &output_results,
        );
    }

    #[test]
    fn converted_directory_cannot_be_created() {
        let source_file = PathBuf::from("source.mp3");
        let destination_directory = PathBuf::from("destination");
        let source_destination_directory = PathBuf::from("source_destination");

        let setting = Setting {
            destination_directory: destination_directory.clone(),
            move_source_file_to: Some(source_destination_directory),
            source_files: vec![source_file],
        };

        let runner = {
            let mut runner = MockConvertForItunesRunner::new();

            runner
                .expect_create_destination_directory()
                .with(predicate::eq(destination_directory.clone()))
                .returning(|_| Err(io::Error::new(io::ErrorKind::Other, "error")));
            runner.expect_convert_all().never();
            runner.expect_move_files().never();

            runner
        };

        let error = convert_for_itunes_on_runner(&setting, runner).unwrap_err();

        assert!(matches!(
            error,
            ConvertForITunesError::DirectoryCannotBeCreated(path, error)
            if path == destination_directory && error.kind() == io::ErrorKind::Other
        ));
    }

    #[test]
    fn source_destination_directory_cannot_be_created() {
        let source_file = PathBuf::from("source.mp3");
        let destination_directory = PathBuf::from("destination");
        let source_destination_directory = PathBuf::from("source_destination");

        let setting = Setting {
            destination_directory: destination_directory.clone(),
            move_source_file_to: Some(source_destination_directory.clone()),
            source_files: vec![source_file],
        };

        let runner = {
            let mut runner = MockConvertForItunesRunner::new();

            runner
                .expect_create_destination_directory()
                .with(predicate::eq(destination_directory.clone()))
                .returning(|_| Ok(()));
            runner
                .expect_create_destination_directory()
                .with(predicate::eq(source_destination_directory.clone()))
                .returning(|_| Err(io::Error::new(io::ErrorKind::Other, "error")));
            runner.expect_convert_all().never();
            runner.expect_move_files().never();

            runner
        };

        let error = convert_for_itunes_on_runner(&setting, runner).unwrap_err();

        assert!(matches!(
            error,
            ConvertForITunesError::DirectoryCannotBeCreated(path, error)
            if path == source_destination_directory && error.kind() == io::ErrorKind::Other
        ));
    }

    #[test]
    fn conversion_is_failed() {
        let source_file = PathBuf::from("source.mp3");
        let destination_directory = PathBuf::from("destination");
        let source_destination_directory = PathBuf::from("source_destination");

        let setting = Setting {
            destination_directory: destination_directory.clone(),
            move_source_file_to: Some(source_destination_directory.clone()),
            source_files: vec![source_file],
        };

        let runner = {
            let mut runner = MockConvertForItunesRunner::new();

            runner
                .expect_create_destination_directory()
                .returning(|_| Ok(()));
            runner
                .expect_convert_all()
                .returning(|_, _| Err(ConversionError::Unknown));
            runner.expect_move_files().never();

            runner
        };

        let error = convert_for_itunes_on_runner(&setting, runner).unwrap_err();

        assert!(matches!(
            error,
            ConvertForITunesError::ConversionError(ConversionError::Unknown)
        ));
    }

    #[test]
    fn moving_is_failed() {
        let source_file = PathBuf::from("source.mp3");
        let destination_directory = PathBuf::from("destination");
        let source_destination_directory = PathBuf::from("source_destination");

        let setting = Setting {
            destination_directory: destination_directory.clone(),
            move_source_file_to: Some(source_destination_directory.clone()),
            source_files: vec![source_file],
        };

        let runner = {
            let mut runner = MockConvertForItunesRunner::new();

            runner
                .expect_create_destination_directory()
                .returning(|_| Ok(()));
            runner.expect_convert_all().returning({
                |source_files, destination| {
                    Ok(vec![ConvertedFile {
                        source: source_files[0].to_path_buf(),
                        destination: destination.to_path_buf(),
                    }])
                }
            });
            runner.expect_move_files().returning(|_, _| {
                Err(FileMovingError::IoError(io::Error::new(
                    io::ErrorKind::Other,
                    "error",
                )))
            });

            runner
        };

        let error = convert_for_itunes_on_runner(&setting, runner).unwrap_err();

        assert!(matches!(
            error,
            ConvertForITunesError::MovingSourceFileIsFailed(FileMovingError::IoError(error))
            if error.kind() == io::ErrorKind::Other
        ));
    }

    #[test]
    fn parse_command_line_without_arguments() {
        let arguments: &[&OsStr] = &[OsStr::new("command")];

        let error = Setting::try_parse_from(arguments).unwrap_err();

        assert_eq!(
            clap::error::ErrorKind::MissingRequiredArgument,
            error.kind()
        );
    }

    #[test]
    fn parse_command_line_with_available_source_files() {
        let destination_directory = PathBuf::from("destination_directory");
        let source_file = NamedTempFile::new().unwrap();

        let arguments = &[
            OsStr::new("command"),
            destination_directory.as_os_str(),
            source_file.path().as_os_str(),
        ];

        let setting = Setting::try_parse_from(arguments).unwrap();

        assert!(setting.move_source_file_to.is_none());
        assert_eq!(destination_directory, setting.destination_directory);

        assert_eq!(1, setting.source_files.len());
        assert_eq!(source_file.path(), setting.source_files[0]);
    }

    #[test]
    fn parse_command_line_with_unavailable_source_files() {
        let destination_directory = PathBuf::from("destination_directory");

        let source_directory = tempdir().unwrap();
        let source_file = source_directory.path().join("unavailable.mp3");

        let arguments = &[
            OsStr::new("command"),
            destination_directory.as_os_str(),
            source_file.as_os_str(),
        ];

        let error = Setting::try_parse_from(arguments).unwrap_err();

        assert_eq!(clap::error::ErrorKind::ValueValidation, error.kind());
    }

    #[test]
    fn parse_command_line_without_destination_directory() {
        let source_directory = tempdir().unwrap();
        let source_file1 = NamedTempFile::new_in(&source_directory).unwrap();
        let source_file2 = NamedTempFile::new_in(&source_directory).unwrap();

        let arguments = &[
            OsStr::new("command"),
            source_file1.path().as_os_str(),
            source_file2.path().as_os_str(),
        ];

        let error = Setting::try_parse_from(arguments).unwrap_err();

        assert_eq!(clap::error::ErrorKind::ValueValidation, error.kind());
    }

    #[test]
    fn parse_command_line_with_moving_source_file() {
        let moving_source_file_directory = PathBuf::from("moving_source_file");
        let destination_directory = PathBuf::from("destination");
        let source_file = NamedTempFile::new().unwrap();

        let arguments = &[
            OsStr::new("command"),
            OsStr::new("--move-source-file-to"),
            moving_source_file_directory.as_os_str(),
            destination_directory.as_os_str(),
            source_file.path().as_os_str(),
        ];

        let setting = Setting::try_parse_from(arguments).unwrap();

        assert_eq!(
            moving_source_file_directory,
            setting.move_source_file_to.unwrap()
        );
    }
}
