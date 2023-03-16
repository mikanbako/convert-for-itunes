use std::path::Path;

use crate::{conversion_error::ConversionError, element::common};

// NOTE: Check applying replaygain
//
// ffmpeg -i <input> -filter:a "volume=replaygain=album" <output>.wav

pub fn convert_to_wav(source_file: &Path, destination_file: &Path) -> Result<(), ConversionError> {
    const COMMAND_NAME: &str = "ffmpeg";

    let mut ffmpeg = common::get_command(COMMAND_NAME)?;
    let command = ffmpeg
        .arg("-loglevel")
        .arg("error")
        .arg("-i")
        .arg(source_file)
        .arg("-filter:a")
        .arg("volume=replaygain=album")
        .arg(destination_file);

    common::run_command(command, COMMAND_NAME)
}
