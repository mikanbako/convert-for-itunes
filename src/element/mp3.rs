// SPDX-FileCopyrightText: 2024 Keita Kita <maoutwo@gmail.com>
//
// SPDX-License-Identifier: MIT

//! An element for MP3 files.

use std::path::Path;

use crate::conversion_error::ConversionError;

use super::{aac::AacGain, common, lame, MetadataWriter, Mp3Converter};

/// Whether a file is an MP3 file.
pub fn is_mp3<P: AsRef<Path>>(file: P) -> bool {
    common::has_extension("mp3", file)
}

/// [`AacGain`] is used as analyzer because aacgain can treat MP3 files.
pub type Mp3Gain = AacGain;

/// Re-converts an MP3 file.
///
/// LAME is used.
pub struct Mp3Reconverter;

impl Mp3Converter for Mp3Reconverter {
    fn convert(&self, source_file: &Path, destination_file: &Path) -> Result<(), ConversionError> {
        lame::convert_to_mp3(source_file, destination_file)
    }
}

impl MetadataWriter for Mp3Reconverter {
    fn copy_metadata(&self, _: &Path, _: &Path) -> Result<(), ConversionError> {
        // Metadata are copied by lame in convert().
        Ok(())
    }
}
