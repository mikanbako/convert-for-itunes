use std::path::Path;

use crate::{conversion_error::ConversionError, element::common};

// MP3 to MP3, WAV to MP3

pub fn convert_to_mp3(source_file: &Path, destination_file: &Path) -> Result<(), ConversionError> {
    const COMMAND_NAME: &str = "lame";

    let mut lame = common::get_command(COMMAND_NAME)?;
    let command = lame
        .arg("-V5")
        .arg("--silent")
        .arg(source_file)
        .arg(destination_file);

    common::run_command(command, COMMAND_NAME)
}
