use std::path::Path;

use lofty::{id3::v2::ID3v2Tag, read_from_path, ItemKey, TagExt, TaggedFile, TaggedFileExt};

use crate::conversion_error::ConversionError;

use super::MetadataWriter;

fn read_tagged_file(source_file: &Path) -> Result<TaggedFile, ConversionError> {
    match read_from_path(source_file) {
        Ok(tagged_file) => Ok(tagged_file),
        Err(error) => Err(ConversionError::CannotReadMetadata {
            cause: error.to_string(),
        }),
    }
}

pub struct LoftyMetadataWriter;

impl LoftyMetadataWriter {
    const EXCEPTED_TAG_ITEMS: &[ItemKey] = &[
        ItemKey::ReplayGainAlbumGain,
        ItemKey::ReplayGainAlbumPeak,
        ItemKey::ReplayGainTrackGain,
        ItemKey::ReplayGainTrackPeak,
    ];
}

impl MetadataWriter for LoftyMetadataWriter {
    fn copy_metadata(&self, source_file: &Path, target_file: &Path) -> Result<(), ConversionError> {
        let source_tagged_file = read_tagged_file(source_file)?;
        let Some(source_tag) = source_tagged_file.primary_tag() else {
            return Ok(());
        };

        let mut tag = source_tag.to_owned();

        for key in Self::EXCEPTED_TAG_ITEMS {
            tag.remove_key(key);
        }

        let id3v2_tag: ID3v2Tag = tag.into();

        match id3v2_tag.save_to_path(target_file) {
            Ok(_) => Ok(()),
            Err(error) => Err(ConversionError::CannotWriteMetadata {
                cause: error.to_string(),
            }),
        }
    }
}
