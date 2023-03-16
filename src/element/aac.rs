use std::{
    path::{Path, PathBuf},
    process::Stdio,
};

use super::{common, ffmpeg, lame, Analyzer, Mp3Converter};

pub fn is_aac<P: AsRef<Path>>(file: P) -> bool {
    common::has_extension("m4a", file)
}

pub struct AacGain;

impl Analyzer for AacGain {
    fn analyze<'a>(
        &self,
        source_paths_in_album: &[&'a std::path::Path],
    ) -> Result<Vec<PathBuf>, crate::conversion_error::ConversionError> {
        const COMMAND_NAME: &str = "aacgain";

        let mut aacgain = common::get_command(COMMAND_NAME)?;
        let command = aacgain
            .arg("-q")
            .arg("-r")
            .arg("-a")
            .args(source_paths_in_album)
            .stdout(Stdio::null());

        common::run_command(command, COMMAND_NAME)?;

        Ok(common::create_path_bufs(source_paths_in_album))
    }
}

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
