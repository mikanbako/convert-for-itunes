// SPDX-FileCopyrightText: 2024 Keita Kita <maoutwo@gmail.com>
//
// SPDX-License-Identifier: MIT

use std::{
    ffi::OsString,
    fs::File,
    io::{self, Write},
    path::{Path, PathBuf},
};

use clap::Parser;
use convert_for_itunes::{
    conversion_error::ConversionError,
    convert_for_itunes::{convert_for_itunes, ConvertForITunesError, OutputResult, Setting},
};
use lofty::{Accessor, TaggedFileExt};
use tempfile::{tempdir, TempDir};

mod common;

#[test]
fn no_arguments_are_error() {
    let arguments: [OsString; 0] = [];

    assert!(Setting::try_parse_from(arguments).is_err())
}

#[derive(Debug, Default)]
struct Input {
    pub arguments: Vec<OsString>,
    pub source_files: Vec<PathBuf>,
    pub destination_directory: Option<TempDir>,
    pub moving_destination_directory: Option<TempDir>,
}

impl Input {
    pub fn builder() -> InputBuilder {
        InputBuilder::default()
    }

    pub fn get_source_file(&self, filename: &str) -> PathBuf {
        self.source_files
            .iter()
            .find_map(|path| {
                if path.file_name().unwrap() == filename {
                    Some(path.to_path_buf())
                } else {
                    None
                }
            })
            .unwrap()
    }
}

struct InputBuilder {
    input: Input,
}

impl Default for InputBuilder {
    fn default() -> Self {
        let mut input = Input::default();

        input.arguments.push("convert-for-itunes".into());

        InputBuilder { input }
    }
}

impl InputBuilder {
    fn convert<P: AsRef<Path>>(
        mut self,
        source_files: &[P],
        destination_directory: TempDir,
    ) -> InputBuilder {
        self.input.source_files = source_files
            .iter()
            .map(|source_file| source_file.as_ref().to_path_buf())
            .collect();
        self.input.destination_directory = Some(destination_directory);

        self
    }

    fn move_sources(mut self) -> InputBuilder {
        self.input.moving_destination_directory = Some(tempdir().unwrap());

        self
    }

    fn build(mut self) -> Input {
        if let Some(ref moving_destination_directory) = self.input.moving_destination_directory {
            self.input.arguments.push(OsString::from("-m"));
            self.input.arguments.push(
                moving_destination_directory
                    .path()
                    .as_os_str()
                    .to_os_string(),
            );
        }

        if let Some(ref destination_directory) = self.input.destination_directory {
            self.input
                .arguments
                .push(destination_directory.path().as_os_str().to_os_string());
        }

        self.input.arguments.extend(
            self.input
                .source_files
                .iter()
                .map(|path| path.as_os_str().to_os_string()),
        );

        self.input
    }
}

fn get_output_result<'a>(
    source_file: &PathBuf,
    results: &'a [OutputResult],
) -> Option<&'a OutputResult> {
    results.iter().find(|result| &result.source == source_file)
}

fn read_primary_tag<P: AsRef<Path>>(music_file: P) -> lofty::Tag {
    let tagged_file = lofty::read_from_path(music_file).unwrap();

    tagged_file.primary_tag().unwrap().to_owned()
}

fn assert_converted_result(source_filename: &str, input: &Input, results: &[OutputResult]) {
    let source_file = input.get_source_file(source_filename);
    let output_result = get_output_result(&source_file, results).unwrap();

    let source_tag = read_primary_tag(source_file);
    let destination_tag = read_primary_tag(&output_result.converted_destination);

    assert_eq!(
        source_tag.title().unwrap(),
        destination_tag.title().unwrap()
    );
}

fn assert_replaygain_is_analyzed<P: AsRef<Path>>(music_file: P) {
    let tag = read_primary_tag(music_file);

    assert!(tag.get(&lofty::ItemKey::ReplayGainTrackGain).is_some());
    assert!(tag.get(&lofty::ItemKey::ReplayGainAlbumGain).is_some());
}

#[test]
fn convert_only() {
    let source_directory = tempdir().unwrap();
    let source_files =
        common::prepare_test_files_in_directory(&["test1.ogg", "test2.ogg"], &source_directory)
            .unwrap();

    let input = Input::builder()
        .convert(&source_files, tempdir().unwrap())
        .build();

    let setting = Setting::try_parse_from(&input.arguments).unwrap();

    let result = convert_for_itunes(&setting).unwrap();

    assert_converted_result("test1.ogg", &input, &result);
    assert_converted_result("test2.ogg", &input, &result);

    assert_replaygain_is_analyzed(&input.source_files[0]);
    assert_replaygain_is_analyzed(&input.source_files[1]);
}

#[test]
fn convert_and_move() {
    let source_directory = tempdir().unwrap();
    let source_files =
        common::prepare_test_files_in_directory(&["test1.ogg", "test2.ogg"], &source_directory)
            .unwrap();

    let input = Input::builder()
        .convert(&source_files, tempdir().unwrap())
        .move_sources()
        .build();

    let setting = Setting::try_parse_from(input.arguments).unwrap();

    let results = convert_for_itunes(&setting).unwrap();

    assert!(!results[0].source.exists());
    assert!(!results[1].source.exists());

    assert!(results[0].converted_destination.is_file());
    assert!(results[1].converted_destination.is_file());

    assert!(results[0].moving_destination.as_ref().unwrap().is_file());
    assert!(results[1].moving_destination.as_ref().unwrap().is_file());
}

fn create_text_file(filename: &str, directory: &TempDir) -> io::Result<PathBuf> {
    let path = directory.path().join(filename);

    let mut file = File::create(&path)?;

    file.write_all(b"test")?;

    Ok(path)
}

#[test]
fn ignored_files() {
    let source_file_directory = tempdir().unwrap();

    let music_file =
        common::prepare_test_files_in_directory(&["test1.ogg"], source_file_directory.path())
            .unwrap();
    let lowercase_log_file = create_text_file("lowercase_log.log", &source_file_directory).unwrap();
    let uppercase_log_file = create_text_file("UPPERCASE_LOG.LOG", &source_file_directory).unwrap();
    let text_file = create_text_file("text.txt", &source_file_directory).unwrap();

    let input = Input::builder()
        .convert(
            &[
                music_file[0].as_path(),
                lowercase_log_file.as_path(),
                uppercase_log_file.as_path(),
                text_file.as_path(),
            ],
            tempdir().unwrap(),
        )
        .move_sources()
        .build();

    let setting = Setting::try_parse_from(input.arguments).unwrap();

    let results = convert_for_itunes(&setting).unwrap();

    assert_eq!(1, results.len());

    assert_eq!(music_file[0], results[0].source);
    assert!(!results[0].source.exists());
    assert!(lowercase_log_file.exists());
    assert!(uppercase_log_file.exists());
    assert!(text_file.exists());
}

#[test]
fn no_music_files() {
    let destination_directory = tempdir().unwrap();
    let lowercase_log_file = create_text_file("lowercase_log.log", &destination_directory).unwrap();
    let uppercase_log_file = create_text_file("UPPERCASE_LOG.LOG", &destination_directory).unwrap();
    let text_file = create_text_file("text.txt", &destination_directory).unwrap();

    let input = Input::builder()
        .convert(
            &[
                lowercase_log_file.as_path(),
                uppercase_log_file.as_path(),
                text_file.as_path(),
            ],
            destination_directory,
        )
        .move_sources()
        .build();

    let setting = Setting::try_parse_from(input.arguments).unwrap();

    let results = convert_for_itunes(&setting).unwrap();

    assert!(results.is_empty());

    assert!(lowercase_log_file.exists());
    assert!(uppercase_log_file.exists());
    assert!(text_file.exists());
}

#[test]
fn convert_with_same_source_and_destination_directory() {
    let source_directory = tempdir().unwrap();
    let source_directory_path = source_directory.path().to_path_buf();
    let source_files =
        common::prepare_test_files_in_directory(&["test1.ogg"], &source_directory).unwrap();

    let input = Input::builder()
        .convert(&source_files, source_directory)
        .build();

    let setting = Setting::try_parse_from(input.arguments).unwrap();

    let error = convert_for_itunes(&setting).unwrap_err();

    assert!(matches!(error,
            ConvertForITunesError::ConversionError(
                ConversionError::SourceFileInDestinationDirectory { source_file, destination_directory })
            if source_file == source_files[0] && destination_directory == source_directory_path))
}

fn convert_and_move_with_single(filename: &str) {
    let source_directory = tempdir().unwrap();
    let source_files =
        common::prepare_test_files_in_directory(&[filename], &source_directory).unwrap();

    let input = Input::builder()
        .convert(&source_files, tempdir().unwrap())
        .move_sources()
        .build();

    let setting = Setting::try_parse_from(input.arguments).unwrap();

    let results = convert_for_itunes(&setting).unwrap();

    assert_eq!(1, results.len());

    assert!(!results[0].source.exists());

    assert!(results[0].converted_destination.is_file());
    assert!(results[0].moving_destination.as_ref().unwrap().is_file());
}

#[test]
fn convert_and_move_ogg_vorbis() {
    convert_and_move_with_single("test1.ogg");
}

#[test]
fn convert_and_move_flac() {
    convert_and_move_with_single("test1.flac");
}

#[test]
fn convert_and_move_mp3() {
    convert_and_move_with_single("test1.mp3");
}

#[test]
fn convert_and_move_aac() {
    convert_and_move_with_single("test1.m4a");
}
