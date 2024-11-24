// SPDX-FileCopyrightText: 2024 Keita Kita <maoutwo@gmail.com>
//
// SPDX-License-Identifier: MIT

mod common;

use convert_for_itunes::metadata::{self, LoftyMetadataParser, MetadataParser};
use lofty::{file::TaggedFileExt, tag::Accessor};

#[test]
fn copy_metadata() {
    let (files, _directory) =
        common::prepare_test_files(&["test1.ogg", "no_metadata.mp3"]).unwrap();

    let source_file = &files[0];
    let target_file = &files[1];

    metadata::copy_metadata(source_file, target_file).unwrap();

    let tagged_file = lofty::read_from_path(target_file).unwrap();
    let actual_tag = tagged_file.primary_tag().unwrap();

    assert_eq!(1, actual_tag.track().unwrap());
    assert_eq!(4, actual_tag.track_total().unwrap());

    assert_eq!(2, actual_tag.disk().unwrap());
    assert_eq!(3, actual_tag.disk_total().unwrap());
}

#[test]
fn copy_metadata_with_no_medatada_file() {
    let (files, _directory) =
        common::prepare_test_files(&["no_metadata.mp3", "no_metadata.mp3"]).unwrap();

    let source_file = &files[0];
    let target_file = &files[1];

    metadata::copy_metadata(source_file, target_file).unwrap();

    let tagged_file = lofty::read_from_path(target_file).unwrap();

    assert!(tagged_file.primary_tag().is_none());
}

#[test]
fn copy_metadata_with_number_and_total() {
    let (files, _directory) = common::prepare_test_files(&[
        "move_test_track_and_disk_number_has_total.ogg",
        "no_metadata.mp3",
    ])
    .unwrap();

    let source_file = &files[0];
    let target_file = &files[1];

    metadata::copy_metadata(source_file, target_file).unwrap();

    let tagged_file = lofty::read_from_path(target_file).unwrap();
    let actual_tag = tagged_file.primary_tag().unwrap();

    assert_eq!(1, actual_tag.track().unwrap());
    assert_eq!(10, actual_tag.track_total().unwrap());

    assert_eq!(2, actual_tag.disk().unwrap());
    assert_eq!(3, actual_tag.disk_total().unwrap());
}

#[test]
fn parse_album_artist() {
    let (source_file, _directory) =
        common::prepare_test_file("album_artist_key_with_space.ogg").unwrap();

    let metadata = LoftyMetadataParser.parse(&source_file).unwrap();

    assert_eq!("album artist", metadata.album_artist.unwrap());
}
