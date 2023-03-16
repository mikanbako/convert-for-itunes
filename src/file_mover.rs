use std::{
    io,
    path::{Path, PathBuf},
};

use anyhow::Result;
use lofty::{Accessor, ItemKey, Tag, TaggedFileExt};
use thiserror::Error;

#[cfg_attr(test, mockall::automock)]
trait MusicMetadata {
    fn get_album_name<'a>(&'a self) -> Option<&'a str>;

    fn get_album_artist<'a>(&'a self) -> Option<&'a str>;

    fn get_track_name<'a>(&'a self) -> Option<&'a str>;

    fn get_track_number(&self) -> Option<u32>;

    fn get_disk_number(&self) -> Option<u32>;
}

#[cfg_attr(test, mockall::automock)]
trait MetadataFactory {
    fn create_metadata(&self, file: &Path) -> Result<Box<dyn MusicMetadata>>;
}

struct LoftyMusicMetadata {
    album_name: Option<String>,
    album_artist: Option<String>,
    track_name: Option<String>,
    track_number: Option<u32>,
    disk_number: Option<u32>,
}

impl LoftyMusicMetadata {
    fn new(path: &Path) -> Result<Self> {
        let tagged_file = lofty::read_from_path(path)?;
        let tag = tagged_file.primary_tag();

        Ok(LoftyMusicMetadata {
            album_name: tag.and_then(|tag| tag.album().map(|album| album.into_owned())),
            album_artist: tag.and_then(LoftyMusicMetadata::get_album_artist),
            track_name: tag.and_then(|tag| tag.title().map(|title| title.into_owned())),
            track_number: tag.and_then(|tag| tag.track()),
            disk_number: tag.and_then(|tag| tag.disk()),
        })
    }

    fn get_album_artist(tag: &Tag) -> Option<String> {
        let album_artist = tag
            .get(&ItemKey::AlbumArtist)
            .and_then(|value| value.value().text());

        match album_artist {
            Some(album_artist) if !album_artist.is_empty() => Some(album_artist.to_owned()),
            _ => tag.artist().map(|artist| artist.into_owned()),
        }
    }
}

impl MusicMetadata for LoftyMusicMetadata {
    fn get_album_name(&self) -> Option<&str> {
        self.album_name.as_deref()
    }

    fn get_album_artist(&self) -> Option<&str> {
        self.album_artist.as_deref()
    }

    fn get_track_name(&self) -> Option<&str> {
        self.track_name.as_deref()
    }

    fn get_track_number(&self) -> Option<u32> {
        self.track_number
    }

    fn get_disk_number(&self) -> Option<u32> {
        self.disk_number
    }
}

struct LoftyMusicMetadataFactory;

impl MetadataFactory for LoftyMusicMetadataFactory {
    fn create_metadata(&self, file: &Path) -> Result<Box<dyn MusicMetadata>> {
        let metadata = LoftyMusicMetadata::new(file)?;

        Ok(Box::new(metadata))
    }
}

pub struct FileMover {
    destination_directory: PathBuf,
    metadata_factory: Box<dyn MetadataFactory>,
}

#[derive(Error, Debug)]
pub enum FileMovingError {
    #[error("Name is invalid: {0}")]
    InvalidName(String),

    #[error("I/O error")]
    IoError(#[from] io::Error),
}

impl FileMover {
    fn new_with_metadata_factory<'a, T: AsRef<Path> + 'a>(
        destination_directory: T,
        metadata_factory: Box<dyn MetadataFactory>,
    ) -> Self {
        FileMover {
            destination_directory: destination_directory.as_ref().to_path_buf(),
            metadata_factory,
        }
    }

    pub fn new<T: AsRef<Path>>(destination_directory: T) -> Self {
        FileMover::new_with_metadata_factory(
            destination_directory,
            Box::new(LoftyMusicMetadataFactory),
        )
    }

    pub fn move_files<T: AsRef<Path>>(self, target_files: &[T]) -> Result<(), FileMovingError> {
        Err(FileMovingError::InvalidName("a".to_owned()))
    }
}

#[cfg(test)]
mod tests {
    use tempfile::{tempdir, TempDir};
    use test_context::{test_context, TestContext};

    use super::{FileMover, MetadataFactory, MockMetadataFactory, MockMusicMetadata};

    struct TempDirectoryContext {
        source_directory: TempDir,
        destination_directory: TempDir,
    }

    impl TempDirectoryContext {
        fn create_file_mover(&self, metadata_factory: Box<dyn MetadataFactory>) -> FileMover {
            FileMover::new_with_metadata_factory(
                self.destination_directory.path(),
                metadata_factory,
            )
        }
    }

    impl TestContext for TempDirectoryContext {
        fn setup() -> Self {
            TempDirectoryContext {
                source_directory: tempdir().unwrap(),
                destination_directory: tempdir().unwrap(),
            }
        }
    }

    #[test_context(TempDirectoryContext)]
    #[test]
    fn no_target_files(context: &mut TempDirectoryContext) {

        // TODO
    }

    // standard file
    // unknown album
    // unknown artist
    // invalid filename
}
