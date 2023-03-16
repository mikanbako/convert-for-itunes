use std::path::{Path, PathBuf};

use tempfile::TempDir;

use crate::conversion_error::ConversionError;

use super::{common, lame, Analyzer, Mp3Converter};

pub fn is_flac<P: AsRef<Path>>(file: P) -> bool {
    common::has_extension("flac", file)
}

pub fn apply_replaygain(flac_files: &[&Path]) -> Result<(), ConversionError> {
    const COMMAND_NAME: &str = "metaflac";

    let mut metafrac = common::get_command(COMMAND_NAME)?;
    let command = metafrac.arg("--add-replay-gain").args(flac_files);

    common::run_command(command, COMMAND_NAME)
}

pub struct MetaFlac;

impl Analyzer for MetaFlac {
    fn analyze(&self, source_paths_in_album: &[&Path]) -> Result<Vec<PathBuf>, ConversionError> {
        const COMMAND_NAME: &str = "metaflac";

        let mut metafrac = common::get_command(COMMAND_NAME)?;
        let command = metafrac
            .arg("--add-replay-gain")
            .args(source_paths_in_album);

        common::run_command(command, COMMAND_NAME)?;

        Ok(common::create_path_bufs(source_paths_in_album))
    }
}

pub fn convert_wav_to_flac(
    source_file: &Path,
    destination_path: &Path,
) -> Result<(), ConversionError> {
    const COMMAND_NAME: &str = "flac";

    let mut flac = common::get_command(COMMAND_NAME)?;
    let command = flac
        .arg("-s")
        .arg("--fast")
        .arg("-o")
        .arg(destination_path)
        .arg(source_file);

    common::run_command(command, COMMAND_NAME)
}

pub struct FlacToMp3Converter;

impl FlacToMp3Converter {
    fn convert_to_wav(&self, source_file: &Path) -> Result<(PathBuf, TempDir), ConversionError> {
        const COMMAND_NAME: &str = "flac";

        let (wav_file_path, temporary_directory) = common::create_temporary_wav_file_path()?;

        let mut flac = common::get_command(COMMAND_NAME)?;
        let command = flac
            .arg("--totally-silent")
            .arg("-d")
            .arg("--apply-replaygain-which-is-not-lossless")
            .arg("-o")
            .arg(&wav_file_path)
            .arg(source_file);

        common::run_command(command, COMMAND_NAME)?;

        Ok((wav_file_path, temporary_directory))
    }
}

impl Mp3Converter for FlacToMp3Converter {
    fn convert(&self, source_file: &Path, destination_file: &Path) -> Result<(), ConversionError> {
        let (wav_file, _temprary_directory) = self.convert_to_wav(source_file)?;

        lame::convert_to_mp3(&wav_file, destination_file)
    }
}
