// SPDX-FileCopyrightText: 2024 Keita Kita <maoutwo@gmail.com>
//
// SPDX-License-Identifier: MIT

//! Metadata of music files.

use std::{
    any::type_name,
    fmt::{self, Debug},
    path::Path,
};

use anyhow::Result;
use lofty::{id3::v2::Id3v2Tag, Accessor, ItemKey, Tag, TagExt, TaggedFile, TaggedFileExt};

use crate::conversion_error::ConversionError;

/// Metadata of a music file.
///
/// The artist of a track is not treated because it is not used for locating music files.
#[derive(Debug, Clone)]
pub struct MusicMetadata {
    pub album_name: Option<String>,

    pub album_artist: Option<String>,

    pub track_name: Option<String>,

    pub track_number: Option<u32>,

    pub disk_number: Option<u32>,
}

/// Parses metadata.
#[cfg_attr(test, mockall::automock)]
pub trait MetadataParser {
    /// Prases the metadata of a music file.
    fn parse(&self, file: &Path) -> Result<MusicMetadata>;
}

impl fmt::Debug for dyn MetadataParser {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", type_name::<Self>())
    }
}

fn get_number_pair(text: &str) -> (Option<u32>, Option<u32>) {
    text.split_once('/')
        .map(|text| {
            (
                text.0.parse::<u32>().map(Some).unwrap_or(None),
                text.1.parse::<u32>().map(Some).unwrap_or(None),
            )
        })
        .unwrap_or((None, None))
}

fn get_number_pair_from_tag(key: &ItemKey, tag: &Tag) -> (Option<u32>, Option<u32>) {
    tag.get_string(key)
        .map(get_number_pair)
        .unwrap_or((None, None))
}

pub struct LoftyMetadataParser;

impl LoftyMetadataParser {
    fn get_album_artist(tag: &Tag) -> Option<String> {
        tag.get_string(&ItemKey::AlbumArtist)
            .filter(|album_artist| !album_artist.is_empty())
            .or_else(|| {
                // Gets another variant of album artist key.
                //
                // Because ItemKey::AlbumArtist is "ALBUMARTIST" for ogg.
                tag.get_string(&ItemKey::Unknown("ALBUM ARTIST".to_owned()))
                    .filter(|album_artist| !album_artist.is_empty())
            })
            .map(|album_artist| album_artist.to_string())
            .or_else(|| tag.artist().map(|artist| artist.into_owned()))
    }

    fn get_track_number(tag: &Tag) -> Option<u32> {
        tag.track()
            .or_else(|| get_number_pair_from_tag(&ItemKey::TrackNumber, tag).0)
    }

    fn get_disk_number(tag: &Tag) -> Option<u32> {
        tag.disk()
            .or_else(|| get_number_pair_from_tag(&ItemKey::DiscNumber, tag).0)
    }
}

impl MetadataParser for LoftyMetadataParser {
    fn parse(&self, path: &Path) -> Result<MusicMetadata> {
        let tagged_file = lofty::read_from_path(path)?;
        let tag = tagged_file.primary_tag();

        Ok(MusicMetadata {
            album_name: tag.and_then(|tag| tag.album().map(|album| album.into_owned())),
            album_artist: tag.and_then(LoftyMetadataParser::get_album_artist),
            track_name: tag.and_then(|tag| tag.title().map(|title| title.into_owned())),
            track_number: tag.and_then(LoftyMetadataParser::get_track_number),
            disk_number: tag.and_then(LoftyMetadataParser::get_disk_number),
        })
    }
}

fn read_tagged_file(source_file: &Path) -> Result<TaggedFile, ConversionError> {
    lofty::read_from_path(source_file).map_err(|error| ConversionError::CannotReadMetadata {
        cause: error.to_string(),
    })
}

pub fn copy_metadata(source_file: &Path, target_file: &Path) -> Result<(), ConversionError> {
    fn get_no_number_string(key: &ItemKey, tag: &Tag) -> Option<String> {
        tag.get_string(key)
            .filter(|string| string.parse::<u32>().is_err())
            .map(|string| string.to_string())
    }

    const EXCEPTED_TAG_ITEMS: &[ItemKey] = &[
        ItemKey::ReplayGainAlbumGain,
        ItemKey::ReplayGainAlbumPeak,
        ItemKey::ReplayGainTrackGain,
        ItemKey::ReplayGainTrackPeak,
    ];

    let source_tagged_file = read_tagged_file(source_file)?;
    let Some(source_tag) = source_tagged_file.primary_tag() else {
        return Ok(());
    };

    let mut tag = source_tag.to_owned();

    for key in EXCEPTED_TAG_ITEMS {
        tag.remove_key(key);
    }

    if let Some(no_number_track) = get_no_number_string(&ItemKey::TrackNumber, &tag) {
        let (number, total) = get_number_pair(&no_number_track);

        if let Some(number) = number {
            tag.set_track(number);
        }
        if let Some(total) = total {
            tag.set_track_total(total);
        }
    }

    if let Some(no_number_disk) = get_no_number_string(&ItemKey::DiscNumber, &tag) {
        let (number, total) = get_number_pair(&no_number_disk);

        if let Some(number) = number {
            tag.set_disk(number);
        }
        if let Some(total) = total {
            tag.set_disk_total(total);
        }
    }

    let id3v2_tag: Id3v2Tag = tag.into();

    id3v2_tag
        .save_to_path(target_file)
        .map_err(|error| ConversionError::CannotWriteMetadata {
            cause: error.to_string(),
        })
}

#[cfg(test)]
mod tests {
    use crate::metadata::get_number_pair;

    #[test]
    fn track_number_from_track_and_album_number() {
        let number_pair = get_number_pair("1/2");

        assert_eq!(Some(1), number_pair.0);
        assert_eq!(Some(2), number_pair.1);
    }

    #[test]
    fn track_number_from_track_and_album_number_without_album_number() {
        let number_pair = get_number_pair("1/");

        assert_eq!(Some(1), number_pair.0);
        assert!(number_pair.1.is_none());
    }
}
