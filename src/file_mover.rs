// SPDX-FileCopyrightText: 2024 Keita Kita <maoutwo@gmail.com>
//
// SPDX-License-Identifier: MIT

//! Move source files to a directory.

use std::{
    collections::HashMap,
    fs::{self, DirBuilder},
    io,
    path::{Path, PathBuf},
};

use anyhow::Result;
use log::{error, info};
use sanitize_filename;
use thiserror::Error;

use crate::metadata::{LoftyMetadataParser, MetadataParser, MusicMetadata};

/// The result of a moving source file.
#[derive(Debug)]
pub struct MovedFile {
    /// The path of the source.
    pub source: PathBuf,

    /// The path of the destination.
    pub destination: PathBuf,
}

/// Error for [`FileMover`].
#[derive(Error, Debug)]
pub enum FileMovingError {
    #[error("The destination path is invalid.")]
    InvalidDestinationPath(PathBuf, io::Error),

    #[error("I/O error.")]
    IoError(io::Error),

    #[error("Reading metadata is failed.")]
    ReadingMetadataIsFailed(anyhow::Error),

    #[error("Duplicated destination.")]
    DuplicatedDestination(Vec<PathBuf>),
}

/// Moves source files to a directory. They are located by their metadata.
#[derive(Debug)]
pub struct FileMover {
    destination_directory: PathBuf,
    metadata_parser: Box<dyn MetadataParser>,
}

impl FileMover {
    const UNKNOWN_ALBUM: &'static str = "Unknown album";
    const UNKNOWN_ARTIST: &'static str = "Unknown artist";

    fn new_with_factory<'a, T: AsRef<Path> + 'a>(
        destination_directory: T,
        metadata_parser: Box<dyn MetadataParser>,
    ) -> Result<Self, FileMovingError> {
        destination_directory
            .as_ref()
            .canonicalize()
            .map(|destination_directory| FileMover {
                destination_directory,
                metadata_parser,
            })
            .map_err(|error| {
                FileMovingError::InvalidDestinationPath(
                    destination_directory.as_ref().to_path_buf(),
                    error,
                )
            })
    }

    /// Creates [`FileMover`] with a destination directory.
    pub fn new<U: AsRef<Path>>(destination_directory: U) -> Result<Self, FileMovingError> {
        FileMover::new_with_factory(destination_directory, Box::new(LoftyMetadataParser))
    }

    fn sanitize_filename(filename: &str) -> String {
        sanitize_filename::sanitize_with_options(
            filename,
            sanitize_filename::Options {
                windows: true,
                truncate: false,
                replacement: "_",
            },
        )
    }

    fn format_numbers(metadata: &MusicMetadata) -> Option<String> {
        let mut numbers = match (metadata.disk_number, metadata.track_number) {
            (Some(disk_number), Some(track_number)) => {
                Some(format!("{disk_number}-{track_number}"))
            }
            (Some(disk_number), None) => Some(format!("{disk_number}-x")),
            (None, Some(track_number)) => Some(format!("{track_number}")),
            (None, None) => None,
        };

        if let Some(ref mut numbers) = numbers {
            numbers.push_str(". ");
        }

        numbers
    }

    fn create_filename<T: AsRef<Path>>(source_file: T, metadata: &MusicMetadata) -> String {
        let mut filename = String::new();

        if let Some(numbers) = Self::format_numbers(metadata) {
            filename.push_str(numbers.as_str());
        }

        if let Some(ref track_name) = metadata.track_name {
            filename.push_str(track_name);

            if let Some(extension) = source_file.as_ref().extension() {
                filename.push_str(format!(".{}", extension.to_string_lossy()).as_str());
            }
        } else if let Some(file_name) = source_file.as_ref().file_name() {
            filename.push_str(file_name.to_string_lossy().as_ref());
        }

        filename
    }

    fn get_destination_path<T: AsRef<Path>>(
        &self,
        source_file: T,
        metadata: &MusicMetadata,
    ) -> PathBuf {
        let album_artist = metadata
            .album_artist
            .as_ref()
            .map_or(Self::UNKNOWN_ARTIST, |album_artist| album_artist);
        let album_name = metadata
            .album_name
            .as_ref()
            .map_or(Self::UNKNOWN_ALBUM, |album_name| album_name);

        let mut destination_path = self.destination_directory.clone();

        destination_path.push(Self::sanitize_filename(album_artist));
        destination_path.push(Self::sanitize_filename(album_name));
        destination_path.push(Self::sanitize_filename(
            Self::create_filename(source_file, metadata).as_str(),
        ));

        destination_path
    }

    fn move_file_to_destination<T, U>(from: T, to: U) -> io::Result<()>
    where
        T: AsRef<Path>,
        U: AsRef<Path>,
    {
        debug_assert!(to.as_ref().is_absolute());

        if let Some(parent_directory) = to.as_ref().parent() {
            DirBuilder::new().recursive(true).create(parent_directory)?;
        }

        if fs::rename(&from, &to).is_err() {
            fs::copy(&from, &to)?;
            fs::remove_file(&from)?;
        }

        Ok(())
    }

    fn move_file(&self, moving_file: &MovedFile) -> Result<(), FileMovingError> {
        Self::move_file_to_destination(&moving_file.source, &moving_file.destination)
            .map_err(FileMovingError::IoError)
    }

    fn get_moving_file<T: AsRef<Path>>(
        &self,
        target_file: T,
    ) -> Result<MovedFile, FileMovingError> {
        let target_file = target_file.as_ref();

        self.metadata_parser
            .parse(target_file)
            .map(|metadata| MovedFile {
                source: target_file.to_path_buf(),
                destination: self.get_destination_path(target_file, &metadata),
            })
            .map_err(FileMovingError::ReadingMetadataIsFailed)
    }

    fn check_duplication(moving_files: &[MovedFile]) -> Result<(), FileMovingError> {
        let destination_to_sources =
            moving_files
                .iter()
                .fold(HashMap::new(), |mut destination_to_sources, moving_file| {
                    destination_to_sources
                        .entry(moving_file.destination.as_path())
                        .and_modify(|sources: &mut Vec<_>| {
                            sources.push(moving_file.source.as_path())
                        })
                        .or_insert_with(|| vec![moving_file.source.as_path()]);

                    destination_to_sources
                });

        let duplicated_sources: Vec<_> = destination_to_sources
            .iter()
            .filter_map(|destination_and_source| {
                if 1 < destination_and_source.1.len() {
                    Some(destination_and_source.1)
                } else {
                    None
                }
            })
            .flatten()
            .map(|source_path| source_path.to_path_buf())
            .collect();

        if duplicated_sources.is_empty() {
            Ok(())
        } else {
            Err(FileMovingError::DuplicatedDestination(duplicated_sources))
        }
    }

    /// Moves files to a directory.
    pub fn move_files<T: AsRef<Path>>(
        &self,
        moving_files: &[T],
    ) -> Result<Vec<MovedFile>, FileMovingError> {
        let moved_files = moving_files
            .iter()
            .map(|target_file| self.get_moving_file(target_file))
            .collect::<Result<Vec<_>, _>>()?;

        Self::check_duplication(&moved_files)?;

        for moving_file in &moved_files {
            self.move_file(moving_file)
                .inspect(|_| {
                    info!(
                        "{:?} was moved to {:?}",
                        moving_file.source, moving_file.destination
                    )
                })
                .inspect_err(|error| error!("Moving file is failed: {}", error))?;
        }

        Ok(moved_files)
    }
}

#[cfg(test)]
mod tests {
    use std::fs::{remove_dir_all, DirBuilder, File};
    use std::io::ErrorKind;
    use std::path::{Path, PathBuf};

    use anyhow::bail;
    use tempfile::{tempdir, TempDir};
    use test_context::{test_context, TestContext};
    use walkdir::WalkDir;

    use super::{FileMover, FileMovingError, MovedFile};
    use crate::metadata::{MetadataParser, MockMetadataParser, MusicMetadata};

    struct TempDirectoryContext {
        source_directory: TempDir,
        destination_directory_path: PathBuf,
    }

    impl TempDirectoryContext {
        pub fn new() -> TempDirectoryContext {
            TempDirectoryContext {
                source_directory: tempdir().unwrap(),
                destination_directory_path: tempdir().unwrap().into_path().canonicalize().unwrap(),
            }
        }

        pub fn create_file_mover(&self, metadata_factory: Box<dyn MetadataParser>) -> FileMover {
            FileMover::new_with_factory(&self.destination_directory_path, metadata_factory).unwrap()
        }

        pub fn create_source_file<T: AsRef<str>>(&self, filename: T) -> PathBuf {
            let path = self.source_directory.path().join(filename.as_ref());

            File::create(&path).unwrap();

            path
        }

        pub fn create_source_file_with_directory<T, U>(&self, directory: T, filename: U) -> PathBuf
        where
            T: AsRef<str>,
            U: AsRef<str>,
        {
            let directory_path = self.source_directory.path().join(directory.as_ref());

            DirBuilder::new().create(&directory_path).unwrap();

            let file_path = directory_path.join(filename.as_ref());

            File::create(&file_path).unwrap();

            file_path
        }

        pub fn get_destination_files(&self) -> Vec<PathBuf> {
            WalkDir::new(&self.destination_directory_path)
                .into_iter()
                .filter_map(|entry| {
                    let directory_entry = entry.unwrap();

                    if directory_entry.file_type().is_file() {
                        Some(directory_entry.into_path())
                    } else {
                        None
                    }
                })
                .collect()
        }

        pub fn get_path_in_destination<T: AsRef<Path>>(&self, path: T) -> PathBuf {
            path.as_ref()
                .strip_prefix(&self.destination_directory_path)
                .unwrap()
                .to_path_buf()
        }

        pub fn assert_destination_file_available<T: AsRef<Path>>(
            &self,
            source_path: T,
            moved_files: &[MovedFile],
        ) {
            let destination_files: Vec<_> = moved_files
                .iter()
                .filter_map({
                    let source_path = source_path.as_ref();

                    move |moved_file| {
                        if moved_file.source == source_path {
                            Some(moved_file.destination.clone())
                        } else {
                            None
                        }
                    }
                })
                .collect();
            assert_eq!(1, destination_files.len());

            assert!(self.get_destination_files().contains(&destination_files[0]));
        }

        pub fn assert_destination_path<T, U, V, W>(
            &self,
            album_artist: T,
            album: U,
            track: V,
            destination_path: W,
        ) where
            T: AsRef<str>,
            U: AsRef<str>,
            V: AsRef<str>,
            W: AsRef<Path>,
        {
            let destination_path_in_directory = self.get_path_in_destination(destination_path);

            let mut path_iter = destination_path_in_directory.iter();

            assert_eq!(album_artist.as_ref(), path_iter.next().unwrap());
            assert_eq!(album.as_ref(), path_iter.next().unwrap());
            assert_eq!(track.as_ref(), path_iter.next().unwrap());
            assert!(path_iter.next().is_none());
        }

        fn assert_destination<T, U, V, W>(
            &self,
            expected_album_artist: T,
            expected_album: U,
            expected_track: V,
            source_filename: W,
            metadata: &MusicMetadata,
        ) where
            T: AsRef<str>,
            U: AsRef<str>,
            V: AsRef<str>,
            W: AsRef<str>,
        {
            let source_path = self.create_source_file(source_filename);

            let metadata_parser = {
                let mut parser = MockMetadataParser::new();

                parser.expect_parse().return_once({
                    let metadata = metadata.clone();

                    |_| Ok(metadata)
                });

                parser
            };

            let file_mover = self.create_file_mover(Box::new(metadata_parser));
            let moved_files = file_mover.move_files(&[&source_path]).unwrap();

            assert_eq!(1, moved_files.len());
            let moved_file = &moved_files[0];

            let actual_destination_files = self.get_destination_files();

            assert_eq!(1, actual_destination_files.len());
            let actual_destination_path = &actual_destination_files[0];

            assert_eq!(source_path, moved_file.source);
            assert_eq!(&moved_file.destination, actual_destination_path);
            assert!(actual_destination_path.is_file());

            self.assert_destination_path(
                expected_album_artist,
                expected_album,
                expected_track,
                actual_destination_path,
            );
        }

        pub fn assert_same_destinations<T: AsRef<Path>>(
            sources: &[T],
            result: &Result<Vec<MovedFile>, FileMovingError>,
        ) {
            let same_destination_sources =
                if let Err(FileMovingError::DuplicatedDestination(same_destination_sources)) =
                    result
                {
                    same_destination_sources
                } else {
                    panic!(
                        "Result is not FileMovingError::SameDestination: {:?}",
                        result
                    )
                };

            assert_eq!(sources.len(), same_destination_sources.len());
            sources.iter().for_each(|source| {
                assert!(same_destination_sources.contains(&source.as_ref().to_path_buf()))
            });
        }
    }

    impl TestContext for TempDirectoryContext {
        fn setup() -> Self {
            TempDirectoryContext::new()
        }
    }

    impl Drop for TempDirectoryContext {
        fn drop(&mut self) {
            let _ = remove_dir_all(&self.destination_directory_path);
        }
    }

    struct MusicMetadataBuilder {
        metadata: MusicMetadata,
    }

    impl Default for MusicMetadataBuilder {
        fn default() -> Self {
            MusicMetadataBuilder {
                metadata: MusicMetadata {
                    album_artist: None,
                    album_name: None,
                    disk_number: None,
                    track_name: None,
                    track_number: None,
                },
            }
        }
    }

    impl MusicMetadataBuilder {
        pub fn new() -> MusicMetadataBuilder {
            MusicMetadataBuilder::default()
        }

        pub fn album_name<T: AsRef<str>>(mut self, album_name: T) -> Self {
            self.metadata.album_name = Some(album_name.as_ref().to_string());

            self
        }

        pub fn album_artist<T: AsRef<str>>(mut self, album_artist: T) -> Self {
            self.metadata.album_artist = Some(album_artist.as_ref().to_string());

            self
        }

        pub fn track_name<T: AsRef<str>>(mut self, track_name: T) -> Self {
            self.metadata.track_name = Some(track_name.as_ref().to_string());

            self
        }

        pub fn track_number(mut self, track_number: u32) -> Self {
            self.metadata.track_number = Some(track_number);

            self
        }

        pub fn disk_number(mut self, disk_number: u32) -> Self {
            self.metadata.disk_number = Some(disk_number);

            self
        }

        pub fn build(self) -> MusicMetadata {
            self.metadata
        }
    }

    #[test_context(TempDirectoryContext)]
    #[test]
    fn no_target_files(context: &TempDirectoryContext) {
        let file_mover = context.create_file_mover(Box::new(MockMetadataParser::new()));
        let no_target_files: &[&Path] = &[];

        let moved_files = file_mover.move_files(no_target_files);

        assert!(moved_files.unwrap().is_empty());
        assert!(context.get_destination_files().is_empty());
    }

    #[test_context(TempDirectoryContext)]
    #[test]
    fn standard_file(context: &TempDirectoryContext) {
        context.assert_destination(
            "album artist",
            "album name",
            "2-1. track name.mp3",
            "a.mp3",
            &MusicMetadataBuilder::new()
                .album_artist("album artist")
                .album_name("album name")
                .track_name("track name")
                .track_number(1)
                .disk_number(2)
                .build(),
        );
    }

    fn get_destination_file<T: AsRef<Path>>(
        source_path: T,
        moved_files: &[MovedFile],
    ) -> Option<PathBuf> {
        moved_files
            .iter()
            .find(|moved_file| source_path.as_ref() == moved_file.source)
            .map(|moved_file| moved_file.destination.clone())
    }

    #[test_context(TempDirectoryContext)]
    #[test]
    fn multiple_files(context: &TempDirectoryContext) {
        let source1_path = context.create_source_file("a.mp3");
        let source2_path = context.create_source_file("b.mp3");
        let source_paths = &[source1_path.clone(), source2_path.clone()];

        let metadata_parser = {
            let mut parser = MockMetadataParser::new();
            let source1_path = source1_path.clone();
            let source2_path = source2_path.clone();

            parser.expect_parse().returning(move |file| {
                let source1 = source1_path.file_name().unwrap();
                let source2 = source2_path.file_name().unwrap();
                let metadata1 = MusicMetadataBuilder::new()
                    .album_artist("album artist1")
                    .album_name("album name1")
                    .track_name("track name1")
                    .track_number(1)
                    .disk_number(1)
                    .build();
                let metadata2 = MusicMetadataBuilder::new()
                    .album_artist("album artist2")
                    .album_name("album name2")
                    .track_name("track name2")
                    .track_number(2)
                    .disk_number(1)
                    .build();

                match file.file_name().unwrap() {
                    file_name if file_name == source1 => Ok(metadata1),
                    file_name if file_name == source2 => Ok(metadata2),
                    _ => Err(std::io::Error::from(ErrorKind::NotFound).into()),
                }
            });

            parser
        };

        let file_mover = context.create_file_mover(Box::new(metadata_parser));

        let moved_files = file_mover.move_files(source_paths).unwrap();
        assert_eq!(2, moved_files.len());

        context.assert_destination_file_available(&source1_path, &moved_files);
        context.assert_destination_file_available(&source2_path, &moved_files);

        context.assert_destination_path(
            "album artist1",
            "album name1",
            "1-1. track name1.mp3",
            get_destination_file(&source1_path, &moved_files).unwrap(),
        );
        context.assert_destination_path(
            "album artist2",
            "album name2",
            "1-2. track name2.mp3",
            get_destination_file(&source2_path, &moved_files).unwrap(),
        );
    }

    #[test_context(TempDirectoryContext)]
    #[test]
    fn unknown_artist(context: &TempDirectoryContext) {
        context.assert_destination(
            FileMover::UNKNOWN_ARTIST,
            "album",
            "1-2. track.ogg",
            "temp.ogg",
            &MusicMetadataBuilder::new()
                .album_name("album")
                .track_name("track")
                .track_number(2)
                .disk_number(1)
                .build(),
        );
    }

    #[test_context(TempDirectoryContext)]
    #[test]
    fn unknown_album(context: &TempDirectoryContext) {
        context.assert_destination(
            "artist",
            FileMover::UNKNOWN_ALBUM,
            "2-3. track.mp4",
            "temp.mp4",
            &MusicMetadataBuilder::new()
                .album_artist("artist")
                .track_name("track")
                .track_number(3)
                .disk_number(2)
                .build(),
        )
    }

    #[test_context(TempDirectoryContext)]
    #[test]
    fn no_title(context: &TempDirectoryContext) {
        context.assert_destination(
            "artist",
            "album",
            "3-2. temp.mp3",
            "temp.mp3",
            &MusicMetadataBuilder::new()
                .album_artist("artist")
                .album_name("album")
                .track_number(2)
                .disk_number(3)
                .build(),
        )
    }

    #[test_context(TempDirectoryContext)]
    #[test]
    fn no_track_number(context: &TempDirectoryContext) {
        context.assert_destination(
            "artist",
            "album",
            "3-x. track.mp3",
            "temp.mp3",
            &MusicMetadataBuilder::new()
                .album_artist("artist")
                .album_name("album")
                .track_name("track")
                .disk_number(3)
                .build(),
        )
    }

    #[test_context(TempDirectoryContext)]
    #[test]
    fn no_disk_number(context: &TempDirectoryContext) {
        context.assert_destination(
            "artist",
            "album",
            "2. track.mp3",
            "temp.mp.mp3",
            &MusicMetadataBuilder::new()
                .album_artist("artist")
                .album_name("album")
                .track_name("track")
                .track_number(2)
                .build(),
        )
    }

    #[test_context(TempDirectoryContext)]
    #[test]
    fn no_track_disk_number(context: &TempDirectoryContext) {
        context.assert_destination(
            "artist",
            "album",
            "track.mp3",
            "temp.mp.mp3",
            &MusicMetadataBuilder::new()
                .album_artist("artist")
                .album_name("album")
                .track_name("track")
                .build(),
        )
    }

    #[test_context(TempDirectoryContext)]
    #[test]
    fn no_metadata(context: &TempDirectoryContext) {
        context.assert_destination(
            FileMover::UNKNOWN_ARTIST,
            FileMover::UNKNOWN_ALBUM,
            "source.aac",
            "source.aac",
            &MusicMetadataBuilder::new().build(),
        );
    }

    #[test_context(TempDirectoryContext)]
    #[test]
    fn same_metadata_files(context: &TempDirectoryContext) {
        let metadata_parser = {
            let mut metadata_parser = MockMetadataParser::new();

            metadata_parser.expect_parse().returning(|_| {
                Ok(MusicMetadataBuilder::new()
                    .album_artist("artist")
                    .album_name("album")
                    .track_name("track")
                    .track_number(1)
                    .disk_number(2)
                    .build())
            });

            metadata_parser
        };

        let source_path1 = context.create_source_file("a.mp3");
        let source_path2 = context.create_source_file("b.mp3");
        let sources = &[&source_path1, &source_path2];

        let file_mover = context.create_file_mover(Box::new(metadata_parser));

        let result = file_mover.move_files(sources);

        TempDirectoryContext::assert_same_destinations(sources, &result);
    }

    #[test_context(TempDirectoryContext)]
    #[test]
    fn same_filename_in_different_directories_without_metadata(context: &TempDirectoryContext) {
        let filename = "source.mp3";
        let source_path1 = context.create_source_file_with_directory("a", filename);
        let source_path2 = context.create_source_file_with_directory("b", filename);
        let sources = &[&source_path1, &source_path2];

        let metadata_parser = {
            let mut metadata_parser = MockMetadataParser::new();

            metadata_parser
                .expect_parse()
                .returning(|_| Ok(MusicMetadataBuilder::new().build()));

            metadata_parser
        };

        let file_mover = context.create_file_mover(Box::new(metadata_parser));

        let result = file_mover.move_files(sources);

        TempDirectoryContext::assert_same_destinations(sources, &result);
    }

    #[test_context(TempDirectoryContext)]
    #[test]
    fn same_source_files(context: &TempDirectoryContext) {
        let filename = "source.mp3";
        let source_path1 = context.create_source_file_with_directory("a", filename);
        let source_path2 = context.create_source_file_with_directory("b", filename);
        let sources = &[&source_path1, &source_path2];

        let metadata_parser = {
            let mut metadata_parser = MockMetadataParser::new();

            metadata_parser.expect_parse().returning({
                let source_path1 = source_path1.clone();
                let source_path2 = source_path2.clone();

                move |file| {
                    let metadata = MusicMetadataBuilder::new()
                        .track_number(match file {
                            _ if file == source_path1 => 1,
                            _ if file == source_path2 => 2,
                            _ => panic!("Unexpected source: {:?}", file),
                        })
                        .build();

                    Ok(metadata)
                }
            });

            metadata_parser
        };

        let file_mover = context.create_file_mover(Box::new(metadata_parser));

        let moved_files = file_mover.move_files(sources).unwrap();

        context.assert_destination_file_available(source_path1, &moved_files);
        context.assert_destination_file_available(source_path2, &moved_files);
    }

    #[test_context(TempDirectoryContext)]
    #[test]
    fn invalid_names(context: &TempDirectoryContext) {
        context.assert_destination(
            "art_i_st_",
            "a_l_b_um__",
            "_tra__ck_.mp3",
            "source.mp3",
            &MusicMetadataBuilder::new()
                .album_artist(r"art/i\st.")
                .album_name("a\"l:b*um? ")
                .track_name("?tra<>ck|")
                .build(),
        );
    }

    #[test_context(TempDirectoryContext)]
    #[test]
    fn failed_metadata(context: &TempDirectoryContext) {
        let source_path = context.create_source_file("source.mp3");

        let metadata_parser = {
            let mut metadata_parser = MockMetadataParser::new();

            metadata_parser
                .expect_parse()
                .returning(|_| bail!("metadata error"));

            metadata_parser
        };

        let file_mover = context.create_file_mover(Box::new(metadata_parser));

        let result = file_mover.move_files(&[source_path]);

        assert!(matches!(
            result,
            Err(FileMovingError::ReadingMetadataIsFailed(_))
        ));
    }
}
