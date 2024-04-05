// SPDX-FileCopyrightText: 2024 Keita Kita <maoutwo@gmail.com>
//
// SPDX-License-Identifier: MIT

use std::path::{Path, PathBuf};

use anyhow::Result;
use convert_for_itunes::{
    conversion_error::ConversionError,
    music_converter::{convert_all, ConvertedFile},
};
use tempfile::{tempdir, TempDir};

mod common;

fn join_path(filename: &str, directory: &TempDir) -> PathBuf {
    let mut path = PathBuf::from(directory.path());

    path.push(filename);

    path
}

fn assert_converted_file(
    expected_source: &Path,
    expected_destination_filename: &str,
    destination_directory: &TempDir,
    converted_files: &[ConvertedFile],
) {
    let expected_destination = join_path(expected_destination_filename, destination_directory);

    assert!(
        converted_files
            .iter()
            .any(|converted_file| expected_source == converted_file.source
                && expected_destination == converted_file.destination),
        "expected_source: {}, expected_destination: {}, converted_files: {converted_files:?}",
        expected_source.display(),
        expected_destination.display(),
    )
}

#[test]
fn convert_single() {
    let (source_file, _working_directory) = common::prepare_test_file("test1.ogg").unwrap();
    let destination_directory = tempdir().unwrap();

    let converted_files = convert_all(&[&source_file], destination_directory.path()).unwrap();

    assert_eq!(1, converted_files.len());

    let converted_file = converted_files.first().unwrap();

    assert_converted_file(
        &source_file,
        "test1.mp3",
        &destination_directory,
        &converted_files,
    );

    common::assert_metadata(&source_file, converted_file.destination.as_path());
}

#[test]
fn convert_album() {
    let (files, working_directory) =
        common::prepare_test_files(&["test1.ogg", "test2.ogg"]).unwrap();
    let destination_directory = tempdir().unwrap();

    let converted_files = convert_all(
        files
            .iter()
            .map(PathBuf::as_path)
            .collect::<Vec<&Path>>()
            .as_slice(),
        destination_directory.path(),
    )
    .unwrap();

    assert_eq!(2, converted_files.len());

    let source1 = join_path("test1.ogg", &working_directory);
    let source2 = join_path("test2.ogg", &working_directory);

    assert_converted_file(
        &source1,
        "test1.mp3",
        &destination_directory,
        &converted_files,
    );
    assert_converted_file(
        &source2,
        "test2.mp3",
        &destination_directory,
        &converted_files,
    );

    common::assert_metadata(&source1, &join_path("test1.mp3", &destination_directory));
    common::assert_metadata(&source2, &join_path("test2.mp3", &destination_directory));
}

#[test]
fn convert_wav_album() {
    let (files, working_directory) =
        common::prepare_test_files(&["test1.wav", "test2.wav"]).unwrap();
    let destination_directory = tempdir().unwrap();

    let converted_files = convert_all(
        files
            .iter()
            .map(PathBuf::as_path)
            .collect::<Vec<&Path>>()
            .as_slice(),
        destination_directory.path(),
    )
    .unwrap();

    assert_eq!(2, converted_files.len());

    let source1 = join_path("test1.wav", &working_directory);
    let source2 = join_path("test2.wav", &working_directory);

    assert_converted_file(
        &source1,
        "test1.mp3",
        &destination_directory,
        &converted_files,
    );
    assert_converted_file(
        &source2,
        "test2.mp3",
        &destination_directory,
        &converted_files,
    );
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
