use std::path::{Path, PathBuf};

use crate::conversion_error::ConversionError;

use super::{common, lame, Analyzer, Mp3Converter};

pub fn is_ogg_vorbis<P: AsRef<Path>>(file: P) -> bool {
    common::has_extension("ogg", file)
}

pub struct OggVorbisGain;

impl Analyzer for OggVorbisGain {
    fn analyze(&self, source_paths_in_album: &[&Path]) -> Result<Vec<PathBuf>, ConversionError> {
        const COMMAND_NAME: &str = "vorbisgain";

        let mut vorbisgain = common::get_command(COMMAND_NAME)?;
        let command = vorbisgain.arg("-q").arg("-a").args(source_paths_in_album);

        common::run_command(command, COMMAND_NAME)?;

        Ok(common::create_path_bufs(source_paths_in_album))
    }
}

pub struct OggVorbisToMp3Converter;

impl OggVorbisToMp3Converter {
    fn convert_to_wav(
        &self,
        source_file: &Path,
        destination_wav_path: &Path,
    ) -> Result<(), ConversionError> {
        const COMMAND_NAME: &str = "ogg123";

        let mut ogg123 = common::get_command(COMMAND_NAME)?;
        let command = ogg123
            .arg("-q")
            .arg("-d")
            .arg("wav")
            .arg("-f")
            .arg(destination_wav_path)
            .arg(source_file);

        common::run_command(command, COMMAND_NAME)
    }
}

impl Mp3Converter for OggVorbisToMp3Converter {
    fn convert(&self, source_file: &Path, destination_file: &Path) -> Result<(), ConversionError> {
        let (wav_path, _temporary_directory) = common::create_temporary_wav_file_path()?;

        self.convert_to_wav(source_file, &wav_path)?;
        lame::convert_to_mp3(&wav_path, destination_file)?;

        Ok(())
    }
}
