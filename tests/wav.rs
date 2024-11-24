// SPDX-FileCopyrightText: 2024 Keita Kita <maoutwo@gmail.com>
//
// SPDX-License-Identifier: MIT

use std::path::PathBuf;

use convert_for_itunes::element::{ElementFactory, Elements};
use lofty::file::TaggedFileExt;

mod common;

#[test]
fn analyze_single() {
    common::analyze_single("test1.wav");
}

#[test]
fn analyze_album() {
    common::analyze_album(&["test1.wav", "test2.wav"]);
}

#[test]
fn convert() {
    common::convert_source("test1.wav");
}

#[test]
fn copy_metadata() {
    let (test_file, temporary_directory) = common::prepare_test_file("test1.wav").unwrap();
    let working_directory = tempfile::tempdir().unwrap();
    let destination_path = {
        let mut path = PathBuf::from(temporary_directory.path());

        path.push("destination.mp3");
        path
    };

    let elements = Elements::new(working_directory.path());

    let analyzed_files = elements
        .create_analyzer(&[&test_file])
        .unwrap()
        .analyze(&[&test_file])
        .unwrap();

    elements
        .create_mp3_converter(&analyzed_files[0])
        .unwrap()
        .convert(&analyzed_files[0], &destination_path)
        .unwrap();
    elements
        .create_metadata_writer(&test_file)
        .unwrap()
        .copy_metadata(&test_file, &destination_path)
        .unwrap();

    let source_tagged_file = lofty::read_from_path(test_file).unwrap();
    let destination_tagged_file = lofty::read_from_path(destination_path).unwrap();

    assert!(source_tagged_file.primary_tag().is_none());
    assert!(destination_tagged_file.primary_tag().is_none());
}
