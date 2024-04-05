// SPDX-FileCopyrightText: 2024 Keita Kita <maoutwo@gmail.com>
//
// SPDX-License-Identifier: MIT

//! A module to use LAME.

use std::path::Path;

use crate::{conversion_error::ConversionError, element::common};

/// Converts an WAV file or an MP3 file to MP3 file.
pub fn convert_to_mp3(source_file: &Path, destination_file: &Path) -> Result<(), ConversionError> {
    const COMMAND_NAME: &str = "lame";

    let mut lame = common::get_command(COMMAND_NAME)?;
    let command = lame
        .arg("-V5")
        .arg("--silent")
        .arg(source_file)
        .arg(destination_file);

    common::run_command(command)
}
