// SPDX-FileCopyrightText: 2024 Keita Kita <maoutwo@gmail.com>
//
// SPDX-License-Identifier: MIT

//! A module that treats metadata of music files by [`lofty`].

use std::path::Path;

use crate::{conversion_error::ConversionError, metadata};

use super::MetadataWriter;

/// [`MetadataWriter`] by [`lofty`].
pub struct LoftyMetadataWriter;

impl MetadataWriter for LoftyMetadataWriter {
    fn copy_metadata(&self, source_file: &Path, target_file: &Path) -> Result<(), ConversionError> {
        metadata::copy_metadata(source_file, target_file)
    }
}
