use std::path::{Path, PathBuf};

use crate::conversion_error::ConversionError;

use super::{aac::AacGain, common, lame, Analyzer, MetadataWriter, Mp3Converter};

pub fn is_mp3<P: AsRef<Path>>(file: P) -> bool {
    common::has_extension("mp3", file)
}

pub struct Mp3Gain {
    // aacgain can be used also for MP3 files.
    aacgain: AacGain,
}

impl Mp3Gain {
    pub fn new() -> Self {
        Mp3Gain { aacgain: AacGain }
    }
}

impl Analyzer for Mp3Gain {
    fn analyze<'a>(
        &self,
        source_paths_in_album: &[&'a Path],
    ) -> Result<Vec<PathBuf>, crate::conversion_error::ConversionError> {
        self.aacgain.analyze(source_paths_in_album)
    }
}

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
