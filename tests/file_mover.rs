use std::path::PathBuf;

use convert_for_itunes::file_mover::{FileMover, FileMovingError, MovedFile};
use tempfile::{tempdir, TempDir};

mod common;

fn move_file(
    source_file: PathBuf,
    destination_directory: &TempDir,
) -> Result<MovedFile, FileMovingError> {
    let file_mover = FileMover::new(destination_directory.path())?;

    file_mover
        .move_files(&[source_file])
        .inspect(|moved_files| assert_eq!(1, moved_files.len()))
        .map(|mut moved_files| moved_files.pop().unwrap())
}

fn assert_moved_file(
    parent_directory_name1: Option<&str>,
    parent_directory_name2: Option<&str>,
    filename: &str,
    moved_file: &MovedFile,
    destination_directory: &TempDir,
) {
    let expected_path = {
        let mut path = destination_directory.path().to_path_buf();

        if let Some(directory_name) = parent_directory_name1 {
            path.push(directory_name);
        }
        if let Some(directory_name) = parent_directory_name2 {
            path.push(directory_name);
        }

        path.push(filename);

        path.canonicalize().unwrap_or(path)
    };

    assert_eq!(
        moved_file
            .destination
            .canonicalize()
            .unwrap_or(moved_file.destination.clone()),
        expected_path
    );
    assert!(expected_path.is_file());
}

fn assert_moving(
    test_filename: &str,
    parent_directory: (Option<&str>, Option<&str>),
    expected_filename: &str,
) {
    let (source_file, _source_directory) = common::prepare_test_file(test_filename).unwrap();
    let destination_directory = tempdir().unwrap();

    let moved_file = move_file(source_file, &destination_directory).unwrap();

    assert_moved_file(
        parent_directory.0,
        parent_directory.1,
        expected_filename,
        &moved_file,
        &destination_directory,
    )
}

#[test]
fn move_ogg() {
    assert_moving(
        "move_test.ogg",
        (Some("album_artist"), Some("album")),
        "2-1. title.ogg",
    );
}

#[test]
fn move_flac() {
    assert_moving(
        "move_test.flac",
        (Some("album_artist"), Some("album")),
        "2-1. title.flac",
    );
}

#[test]
fn move_aac() {
    assert_moving(
        "move_test.m4a",
        (Some("album_artist"), Some("album")),
        "2-1. title.m4a",
    );
}

#[test]
fn move_mp3() {
    assert_moving(
        "move_test.mp3",
        (Some("album_artist"), Some("album")),
        "2-1. title.mp3",
    );
}

#[test]
fn move_wav() {
    assert_moving(
        "move_test.wav",
        (Some("Unknown artist"), Some("Unknown album")),
        "move_test.wav",
    );
}

#[test]
fn move_test_track_and_disk_number_has_total() {
    // The track number is x/y.
    assert_moving(
        "move_test_track_and_disk_number_has_total.ogg",
        (Some("album_artist"), Some("album")),
        "2-1. title.ogg",
    );
}

#[test]
fn move_test_without_album_artist() {
    assert_moving(
        "move_test_without_album_artist.ogg",
        (Some("artist"), Some("album")),
        "2-1. title.ogg",
    );
}

#[test]
fn move_test_without_artist() {
    assert_moving(
        "move_test_without_artist.ogg",
        (Some("Unknown artist"), Some("album")),
        "2-1. title.ogg",
    );
}

#[test]
fn move_test_without_album() {
    assert_moving(
        "move_test_without_album.ogg",
        (Some("album_artist"), Some("Unknown album")),
        "2-1. title.ogg",
    );
}

#[test]
fn move_test_without_disk_number() {
    assert_moving(
        "move_test_without_track_and_disk_number.ogg",
        (Some("album_artist"), Some("album")),
        "title.ogg",
    );
}

#[test]
fn move_test_with_compilation() {
    assert_moving(
        "move_test_with_compilation.ogg",
        (Some("album_artist"), Some("album")),
        "2-1. title.ogg",
    );
}
