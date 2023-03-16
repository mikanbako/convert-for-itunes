use std::path::{Path, PathBuf};

use anyhow::Result;
use convert_for_itunes::{conversion_error::ConversionError, music_converter::convert_all};
use tempfile::{tempdir, TempDir};

mod common;

fn join_path(filename: &str, directory: &TempDir) -> PathBuf {
    let mut path = PathBuf::from(directory.path());

    path.push(filename);

    path
}

#[test]
fn convert_single() {
    let (source_file, _working_directory) = common::prepare_test_file("test1.ogg").unwrap();
    let destination_directory = tempdir().unwrap();

    let mp3_files = convert_all(&[&source_file], destination_directory.path()).unwrap();

    assert_eq!(1, mp3_files.len());

    let destination_file = mp3_files.first().unwrap();

    assert_eq!(
        join_path("test1.mp3", &destination_directory).as_path(),
        destination_file
    );

    common::assert_metadata(&source_file, destination_file);
}

#[test]
fn convert_album() {
    let (files, working_directory) =
        common::prepare_test_files(&["test1.ogg", "test2.ogg"]).unwrap();
    let destination_directory = tempdir().unwrap();

    let mp3_files = convert_all(
        files
            .iter()
            .map(PathBuf::as_path)
            .collect::<Vec<&Path>>()
            .as_slice(),
        destination_directory.path(),
    )
    .unwrap();

    assert_eq!(2, mp3_files.len());
    assert!(mp3_files.contains(&join_path("test1.mp3", &destination_directory)));
    assert!(mp3_files.contains(&join_path("test2.mp3", &destination_directory)));

    common::assert_metadata(
        &join_path("test1.ogg", &working_directory),
        &join_path("test1.mp3", &destination_directory),
    );
    common::assert_metadata(
        &join_path("test2.ogg", &working_directory),
        &join_path("test2.mp3", &destination_directory),
    );
}

#[test]
fn convert_wav_album() {
    let (files, _working_directory) =
        common::prepare_test_files(&["test1.wav", "test2.wav"]).unwrap();
    let destination_directory = tempdir().unwrap();

    let mp3_files = convert_all(
        files
            .iter()
            .map(PathBuf::as_path)
            .collect::<Vec<&Path>>()
            .as_slice(),
        destination_directory.path(),
    )
    .unwrap();

    assert_eq!(2, mp3_files.len());
    assert!(mp3_files.contains(&join_path("test1.mp3", &destination_directory)));
    assert!(mp3_files.contains(&join_path("test2.mp3", &destination_directory)));
}

#[test]
fn convert_invalid_file() -> Result<(), ConversionError> {
    let (source_file, _working_directory) = common::prepare_test_file("invalid.ogg").unwrap();
    let destination_directory = tempdir().unwrap();

    let result = convert_all(&[&source_file], destination_directory.path());

    if let Err(ConversionError::CommandFailed { .. }) = result {
        Ok(())
    } else {
        Err(result.err().unwrap())
    }
}
