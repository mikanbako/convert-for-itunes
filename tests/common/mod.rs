use std::path::PathBuf;
use std::{fs, path::Path};

use anyhow::Result;
use convert_for_itunes::element::{ElementFactory, Elements};
use lofty::{Accessor, FileType, ItemKey, LoftyError, Probe, TagExt, TagItem, TaggedFileExt};
use tempfile::{tempdir, TempDir};

fn get_test_file(filename: &str) -> PathBuf {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));

    path.push("tests");
    path.push("resources");
    path.push(filename);

    path
}

pub fn prepare_test_files(all_filenames: &[&str]) -> Result<(Vec<PathBuf>, TempDir)> {
    let directory = tempdir()?;

    let source_and_destinations: Vec<_> = all_filenames
        .iter()
        .map(|filename| (get_test_file(filename), directory.path().join(filename)))
        .collect();

    let mut all_destination_files: Vec<PathBuf> = Vec::new();
    for (source_file, destination_file) in source_and_destinations {
        fs::copy(source_file, &destination_file)?;

        all_destination_files.push(destination_file);
    }

    Ok((all_destination_files, directory))
}

pub fn prepare_test_file(filename: &str) -> Result<(PathBuf, TempDir)> {
    let (mut destination_files, temporary_directory) = prepare_test_files(&[filename])?;

    Ok((destination_files.pop().unwrap(), temporary_directory))
}

fn get_tag_items(file: &Path) -> Result<Vec<TagItem>, LoftyError> {
    let tagged_file = lofty::read_from_path(file)?;

    Ok(tagged_file
        .tags()
        .iter()
        .flat_map(|tag| tag.items())
        .map(|item| item.to_owned())
        .collect())
}

fn is_mp3(path: &Path) -> Result<bool> {
    let probe = Probe::open(path)?.guess_file_type()?;

    let is_mp3 = match probe.file_type() {
        Some(file_type) => file_type == FileType::MPEG,
        None => false,
    };

    Ok(is_mp3)
}

#[allow(dead_code)]
pub fn analyze_single(filename: &str) {
    let working_directory = tempdir().unwrap();
    let (test_file, _temp_directory) = prepare_test_file(filename).unwrap();
    let before_tag_items = get_tag_items(&test_file).unwrap();

    let analyzer = Elements::new(working_directory.path())
        .create_analyzer(&[test_file.as_path()])
        .unwrap();

    let analyzed_files = analyzer.analyze(&[test_file.as_path()]).unwrap();

    let after_tag_items = get_tag_items(&analyzed_files[0]).unwrap();

    assert!(before_tag_items
        .iter()
        .all(|item| after_tag_items.contains(item)));

    assert!(&[
        ItemKey::ReplayGainAlbumGain,
        ItemKey::ReplayGainAlbumPeak,
        ItemKey::ReplayGainTrackGain,
        ItemKey::ReplayGainTrackPeak,
    ]
    .iter()
    .all(|key| after_tag_items.iter().any(|item| item.key() == key)));
}

#[allow(dead_code)]
pub fn analyze_album(filenames: &[&str]) {
    let working_directory = tempdir().unwrap();
    let (test_files, _temp_directory) = prepare_test_files(filenames).unwrap();
    let test_files: Vec<_> = test_files.iter().map(|file| file.as_path()).collect();

    let analyzer = Elements::new(working_directory.path())
        .create_analyzer(&test_files)
        .unwrap();
    let analyzed_files = analyzer.analyze(&test_files).unwrap();

    type TagsInFile = Vec<TagItem>;

    let after_tag_items: Vec<TagsInFile> = analyzed_files
        .iter()
        .map(|file| get_tag_items(file))
        .collect::<Result<Vec<TagsInFile>, LoftyError>>()
        .unwrap();

    assert_eq!(2, after_tag_items.len());

    let (album_peaks, album_gains): (Vec<_>, Vec<_>) = after_tag_items
        .iter()
        .filter_map(|tags| {
            let album_peak = tags
                .iter()
                .find(|tag| tag.key() == &ItemKey::ReplayGainAlbumPeak);
            let album_gain = tags
                .iter()
                .find(|tag| tag.key() == &ItemKey::ReplayGainAlbumGain);

            if let (Some(album_peak), Some(album_gain)) = (album_peak, album_gain) {
                Some((album_peak, album_gain))
            } else {
                None
            }
        })
        .unzip();

    assert_eq!(2, album_peaks.len());
    assert_eq!(2, album_peaks.len());

    assert_eq!(album_peaks[0], album_peaks[1]);
    assert_eq!(album_gains[0], album_gains[1]);
}

#[allow(dead_code)]
pub fn convert_source(test_filename: &str) {
    let working_directory = tempdir().unwrap();
    let (source_file, directory) = prepare_test_file(test_filename).unwrap();

    let destination_file = {
        let mut path = PathBuf::from(directory.path());

        path.push("test_result.mp3");

        path
    };

    let elements = Elements::new(working_directory.path());

    let analyzed_files = elements
        .create_analyzer(&[&source_file])
        .unwrap()
        .analyze(&[&source_file])
        .unwrap();

    elements
        .create_mp3_converter(&analyzed_files[0])
        .unwrap()
        .convert(&analyzed_files[0], &destination_file)
        .unwrap();

    assert!(is_mp3(&destination_file).unwrap());
}

pub fn assert_metadata(source_file: &Path, destination_file: &Path) {
    let source_tagged_file = lofty::read_from_path(source_file).unwrap();
    let source_tag = source_tagged_file.primary_tag().unwrap();
    let destination_tagged_file = lofty::read_from_path(destination_file).unwrap();
    let destination_tag = destination_tagged_file.primary_tag().unwrap();

    assert_eq!(source_tag.title(), destination_tag.title());
    assert_eq!(source_tag.artist(), destination_tag.artist());
    assert_eq!(source_tag.album(), destination_tag.album());
    assert_eq!(source_tag.comment(), destination_tag.comment());
    assert_eq!(source_tag.year(), destination_tag.year());
    assert_eq!(source_tag.track_total(), destination_tag.track_total());
    assert_eq!(source_tag.track(), destination_tag.track());
    assert_eq!(source_tag.genre(), destination_tag.genre());
    assert_eq!(
        source_tag.get_string(&ItemKey::AlbumArtist).unwrap(),
        destination_tag.get_string(&ItemKey::AlbumArtist).unwrap()
    );
    assert_eq!(source_tag.disk(), destination_tag.disk());
    assert_eq!(source_tag.disk_total(), destination_tag.disk_total());

    if let Some(compilation) = source_tag.get_string(&ItemKey::FlagCompilation) {
        assert_eq!(
            compilation,
            destination_tag
                .get_string(&ItemKey::FlagCompilation)
                .unwrap()
        );
    }

    assert!(!destination_tag.contains(&ItemKey::ReplayGainTrackGain));
    assert!(!destination_tag.contains(&ItemKey::ReplayGainTrackPeak));
    assert!(!destination_tag.contains(&ItemKey::ReplayGainAlbumGain));
    assert!(!destination_tag.contains(&ItemKey::ReplayGainAlbumPeak));
}

#[allow(dead_code)]
pub fn copy_metadata(test_filename: &str) {
    let (test_file, temporary_directory) = prepare_test_file(test_filename).unwrap();
    let working_directory = tempfile::tempdir().unwrap();
    let destination_file = {
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
        .convert(&analyzed_files[0], &destination_file)
        .unwrap();
    elements
        .create_metadata_writer(&test_file)
        .unwrap()
        .copy_metadata(&test_file, &destination_file)
        .unwrap();

    assert_metadata(&test_file, &destination_file);
}
