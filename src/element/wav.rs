use std::path::{Path, PathBuf};

use crate::conversion_error::ConversionError;

use super::{common, flac, lame, Analyzer, Mp3Converter};

pub fn is_wav<P: AsRef<Path>>(file: P) -> bool {
    common::has_extension("wav", file)
}

pub struct WavFlacAnalyzer {
    working_directory: PathBuf,
}

impl WavFlacAnalyzer {
    pub fn new(working_directory: &Path) -> Self {
        WavFlacAnalyzer {
            working_directory: working_directory.to_owned(),
        }
    }
}

impl Analyzer for WavFlacAnalyzer {
    fn analyze<'a>(
        &self,
        source_paths_in_album: &[&'a Path],
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

                match flac::convert_wav_to_flac(wav_path, &flac_path) {
                    Ok(_) => Ok(flac_path),
                    Err(error) => Err(error),
                }
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

pub struct WavFlacToMp3Converter;

impl Mp3Converter for WavFlacToMp3Converter {
    fn convert(
        &self,
        source_file: &Path,
        destination_file: &Path,
    ) -> Result<(), crate::conversion_error::ConversionError> {
        lame::convert_to_mp3(source_file, destination_file)
    }
}
