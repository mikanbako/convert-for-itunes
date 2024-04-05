// SPDX-FileCopyrightText: 2024 Keita Kita <maoutwo@gmail.com>
//
// SPDX-License-Identifier: MIT

//! An element for WAV files.

use std::path::{Path, PathBuf};

use crate::conversion_error::ConversionError;

use super::{common, flac, Analyzer};

/// Whether a file is an WAV file.
pub fn is_wav<P: AsRef<Path>>(file: P) -> bool {
    common::has_extension("wav", file)
}

pub struct WavFlacAnalyzer {
    working_directory: PathBuf,
}

/// An [`Analyzer`] for WAV files by converted FLAC files.
///
/// This analyzer converts from WAV files to FLAC files to apply replaygain.
///
/// flac is used.
impl WavFlacAnalyzer {
    pub fn new(working_directory: &Path) -> Self {
        WavFlacAnalyzer {
            working_directory: working_directory.to_owned(),
        }
    }
}

impl Analyzer for WavFlacAnalyzer {
    fn analyze(
        &self,
        source_paths_in_album: &[&Path],
    ) -> Result<Vec<std::path::PathBuf>, ConversionError> {
        let all_flac_files = source_paths_in_album
            .iter()
            .enumerate()
            .map(|(index, wav_path)| {
                let flac_path = {
                    let mut path = self.working_directory.to_path_buf();

                    path.push(format!("source_{}.flac", index));
                    path
                };

                flac::convert_wav_to_flac(wav_path, &flac_path).map(|_| flac_path)
            })
            .collect::<Result<Vec<_>, _>>()?;

        flac::apply_replaygain(
            all_flac_files
                .iter()
                .map(|path| path.as_path())
                .collect::<Vec<_>>()
                .as_slice(),
        )?;

        Ok(all_flac_files)
    }
}
