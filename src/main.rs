use std::{
    fs::create_dir_all,
    path::{Path, PathBuf},
    process::exit,
};

use clap::Parser;

use convert_for_itunes::{file_mover, music_converter::convert_all, utilities};

// TODO dry-run
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct CommandLine {
    /// A directory to move source files.
    #[arg(
        short,
        long,
        value_name = "DIRECTORY",
        help = "A destination directory that contains source files"
    )]
    move_source_file_to: Option<PathBuf>,

    #[arg(
        required = true,
        value_name = "SOURCE_FILE",
        help = "MP3, Flac or Ogg Vorbis files in an album"
    )]
    source_files: Vec<PathBuf>,

    #[arg(
        required = true,
        help = "A destination directory that contains MP3 files"
    )]
    destination_directory: PathBuf,
}

fn create_destination_directory(path: &Path) -> std::io::Result<()> {
    create_dir_all(path)
}

fn main() {
    let cli = CommandLine::parse();

    if let Some(ref moving_destination_directory) = cli.move_source_file_to {
        let result = create_destination_directory(moving_destination_directory);

        if let Err(e) = result {
            eprintln!(
                "{0} cannot be created: {e}",
                moving_destination_directory.display()
            );

            exit(1);
        }
    }

    let result = convert_all(
        utilities::get_paths_from_path_bufs(&utilities::filter_paths(cli.source_files.as_slice()))
            .as_slice(),
        &cli.destination_directory,
    );

    if let Err(error) = result {
        eprintln!("Conversion is failed. {error}");

        exit(1);
    }

    if let Some(ref moving_destination_directory) = cli.move_source_file_to {
        let file_mover = file_mover::FileMover::new(moving_destination_directory);

        if let Err(error) = file_mover.move_files(cli.source_files.as_slice()) {
            eprintln!("Moving files is failed. {error}");

            exit(1);
        }
    }
}
