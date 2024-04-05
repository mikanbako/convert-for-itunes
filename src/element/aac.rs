// SPDX-FileCopyrightText: 2024 Keita Kita <maoutwo@gmail.com>
//
// SPDX-License-Identifier: MIT

//! An element for AAC files.

use std::{
    path::{Path, PathBuf},
    process::Stdio,
};

use super::{common, ffmpeg, lame, Analyzer, Mp3Converter};

/// Whether a `file` is an AAC file.
pub fn is_aac<P: AsRef<Path>>(file: P) -> bool {
    common::has_extension("m4a", file)
}

/// `Analyzer` for AAC.
///
/// aacgain is used.
pub struct AacGain;

impl Analyzer for AacGain {
    fn analyze(
        &self,
        source_paths_in_album: &[&std::path::Path],
    ) -> Result<Vec<PathBuf>, crate::conversion_error::ConversionError> {
        const COMMAND_NAME: &str = "aacgain";

        let mut aacgain = common::get_command(COMMAND_NAME)?;
        let command = aacgain
            .arg("-q")
            .arg("-r")
            .arg("-a")
            .args(source_paths_in_album)
            .stdout(Stdio::null());

        common::run_command(command)?;

        Ok(common::create_path_bufs(source_paths_in_album))
    }
}

/// [`Mp3Converter`] for AAC.
///
/// FFmpeg and LAME are used.
pub struct AacToMp3Converter;

impl Mp3Converter for AacToMp3Converter {
    fn convert(
        &self,
        source_file: &Path,
        destination_file: &Path,
    ) -> Result<(), crate::conversion_error::ConversionError> {
        let (wav_path, _temporary_directory) = common::create_temporary_wav_file_path()?;

        ffmpeg::convert_to_wav(source_file, &wav_path)?;

        lame::convert_to_mp3(&wav_path, destination_file)
    }
}
