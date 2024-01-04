//! Converts musics files.

use std::{
    ffi::OsString,
    fs,
    path::{Path, PathBuf},
};

use log::{error, info};

use crate::{
    conversion_error::ConversionError, element::ElementFactory, element::Elements, utilities,
};

/// The pair of a source music file and the destination file of converted it.
#[derive(Debug)]
pub struct ConvertedFile {
    /// A source music file.
    pub source: PathBuf,

    /// The destination file.
    pub destination: PathBuf,
}

/// Converts music files and output to a directory.
///
/// `source_files_in_album` are music files in a album. They are converted and
/// output to `destination_directory`.
pub fn convert_all(
    source_files_in_album: &[&Path],
    destination_directory: &Path,
) -> Result<Vec<ConvertedFile>, ConversionError> {
    let temporary_directory = utilities::create_temporary_directory()?;
    let elements = Elements::new(temporary_directory.path());

    convert_all_with_factory(source_files_in_album, destination_directory, &elements)
}

fn check_directories_are_unique(
    all_source_files: &[&Path],
    destination_directory: &Path,
) -> Result<(), ConversionError> {
    let canonicalized_destination_directory = destination_directory
        .canonicalize()
        .map_err(|error| ConversionError::IoError { error })?;

    for source_file in all_source_files {
        let canonicalized_source_file = source_file
            .canonicalize()
            .map_err(|error| ConversionError::IoError { error })?;

        if let Some(source_parent_path) = canonicalized_source_file.parent() {
            if source_parent_path == canonicalized_destination_directory {
                return Err(ConversionError::SourceFileInDestinationDirectory {
                    source_file: source_file.to_path_buf(),
                    destination_directory: destination_directory.to_path_buf(),
                });
            }
        } else {
            return Err(ConversionError::NotFile {
                path: source_file.to_path_buf(),
            });
        }
    }

    Ok(())
}

fn check_filenames_are_unique(files: &[&Path]) -> Result<(), ConversionError> {
    files
        .iter()
        .filter_map(|path| path.file_stem().or_else(|| path.file_name()))
        .try_fold(Vec::new(), |mut collected_filenames, filename| {
            let lowercase_filename = filename.to_ascii_lowercase();

            if collected_filenames.contains(&lowercase_filename) {
                Err(ConversionError::DuplicatedFilename {
                    filename: filename.to_string_lossy().into_owned(),
                })
            } else {
                collected_filenames.push(lowercase_filename);

                Ok(collected_filenames)
            }
        })
        .map(|_| ())
}

fn convert_all_with_factory(
    source_files_in_album: &[&Path],
    destination_directory: &Path,
    element_factory: &dyn ElementFactory,
) -> Result<Vec<ConvertedFile>, ConversionError> {
    check_directories_are_unique(source_files_in_album, destination_directory)?;
    check_filenames_are_unique(source_files_in_album)?;

    let analyzed_files = analyze(source_files_in_album, element_factory)?;

    info!("All music files ware analyzed.");

    let converted_results = source_files_in_album
        .iter()
        .zip(analyzed_files.iter())
        .map(|(source_file, analyzed_file)| {
            convert_to_mp3_from(
                source_file,
                analyzed_file,
                destination_directory,
                element_factory,
            )
            .map(|mp3_file| ConvertedFile {
                source: source_file.to_path_buf(),
                destination: mp3_file,
            })
            .inspect(|converted_file| {
                info!(
                    "{:?} was converted to {:?}",
                    converted_file.source, converted_file.destination
                )
            })
            .inspect_err(|error| error!("Conversion was failed: {}", error))
        })
        .collect::<Vec<_>>();

    let destination_files = converted_results
        .iter()
        .filter_map(|result| {
            result
                .as_ref()
                .map(|result| result.destination.clone())
                .ok()
        })
        .collect::<Vec<_>>();

    converted_results
        .into_iter()
        .collect::<Result<_, _>>()
        .inspect_err(|_| {
            info!("Remove converted files because error was occured.");

            remove_files(&destination_files)
        })
}

fn convert_to_mp3_from<T: AsRef<Path>, U: AsRef<Path>, V: AsRef<Path>>(
    source_file: T,
    analyzed_file: U,
    destination_directory: V,
    element_factory: &dyn ElementFactory,
) -> Result<PathBuf, ConversionError> {
    let output_mp3_path = get_output_mp3_path(&source_file, destination_directory)?;

    convert_to_mp3(
        &source_file,
        &analyzed_file,
        &output_mp3_path,
        element_factory,
    )
    .map(|_| output_mp3_path)
}

fn analyze(
    source_files_in_album: &[&Path],
    element_factory: &dyn ElementFactory,
) -> Result<Vec<PathBuf>, ConversionError> {
    let analyzer = element_factory.create_analyzer(source_files_in_album)?;

    analyzer.analyze(source_files_in_album)
}

fn get_output_mp3_path<T: AsRef<Path>, U: AsRef<Path>>(
    source_file: T,
    destination_directory: U,
) -> Result<PathBuf, ConversionError> {
    let source_file = source_file.as_ref();
    let destination_directory = destination_directory.as_ref();

    if !source_file.is_file() {
        return Err(ConversionError::NotFile {
            path: PathBuf::from(source_file),
        });
    }

    if !destination_directory.is_dir() {
        return Err(ConversionError::NotDirectory {
            path: PathBuf::from(destination_directory),
        });
    }

    let destination_filename = {
        let mut filename = OsString::from(source_file.file_stem().unwrap());

        filename.push(".mp3");

        filename
    };

    let mut destination_path = PathBuf::from(destination_directory);

    destination_path.push(destination_filename);

    Ok(destination_path)
}

fn convert_to_mp3<T: AsRef<Path>, U: AsRef<Path>, V: AsRef<Path>>(
    source_file: &T,
    analyzed_file: &U,
    output_mp3_file: &V,
    element_factory: &dyn ElementFactory,
) -> Result<(), ConversionError> {
    let source_path = source_file.as_ref();
    let analyzed_path = analyzed_file.as_ref();
    let output_mp3_path = output_mp3_file.as_ref();

    element_factory
        .create_mp3_converter(analyzed_path)?
        .convert(analyzed_path, output_mp3_path)?;

    let writer = element_factory.create_metadata_writer(source_path)?;

    writer.copy_metadata(source_path, output_mp3_path)
}

fn remove_files(all_paths: &Vec<PathBuf>) {
    for path in all_paths {
        let _ = fs::remove_file(path);
    }
}

#[cfg(test)]
mod tests {
    use tempfile;

    use super::*;

    use std::{fs::File, path::PathBuf, vec};

    use crate::{
        element::{MockAnalyzer, MockElementFactory, MockMetadataWriter, MockMp3Converter},
        utilities,
    };

    #[test]
    fn convert_with_some_sources() {
        let file_a = tempfile::NamedTempFile::new().unwrap();
        let file_b = tempfile::NamedTempFile::new().unwrap();
        let analyzed_file_a = tempfile::NamedTempFile::new().unwrap();
        let analyzed_file_b = tempfile::NamedTempFile::new().unwrap();

        let mut element_factory = MockElementFactory::new();
        let source_files = vec![PathBuf::from(file_a.path()), PathBuf::from(file_b.path())];
        let analyzed_source_files = vec![
            PathBuf::from(analyzed_file_a.path()),
            PathBuf::from(analyzed_file_b.path()),
        ];
        let destination_directory = tempfile::tempdir().unwrap();

        element_factory
            .expect_create_analyzer()
            .withf({
                let expected_source_files_for_factory = source_files.clone();

                move |source_files| source_files == expected_source_files_for_factory
            })
            .times(1)
            .returning({
                let expected_source_files_for_analyzer = source_files.clone();
                let analyzed_files = analyzed_source_files.clone();

                move |_| {
                    let mut analyzer = MockAnalyzer::new();
                    let expected_files = expected_source_files_for_analyzer.clone();
                    let analyzed_files = analyzed_files.clone();

                    analyzer
                        .expect_analyze()
                        .withf(move |source_files| source_files == expected_files)
                        .returning(move |_| Ok(analyzed_files.clone()));

                    Ok(Box::new(analyzer))
                }
            });

        element_factory
            .expect_create_mp3_converter()
            .withf({
                let expected_source_files = analyzed_source_files.clone();

                move |source_file| expected_source_files.contains(&PathBuf::from(source_file))
            })
            .times(source_files.len())
            .returning({
                let source_files_for_converter = analyzed_source_files.clone();

                move |_| {
                    let mut mp3_converter = MockMp3Converter::new();
                    let expected_source_files = source_files_for_converter.clone();

                    mp3_converter
                        .expect_convert()
                        .withf(move |source_file, _| {
                            expected_source_files.contains(&PathBuf::from(source_file))
                        })
                        .returning(|_, _| Ok(()));

                    Ok(Box::new(mp3_converter))
                }
            });

        element_factory
            .expect_create_metadata_writer()
            .withf({
                let expected_source_files = source_files.clone();

                move |source_file| expected_source_files.contains(&PathBuf::from(source_file))
            })
            .times(source_files.len())
            .returning({
                let source_files_for_writer = source_files.clone();

                move |_| {
                    let mut metadata_writer = MockMetadataWriter::new();
                    let expected_source_files = source_files_for_writer.clone();

                    metadata_writer
                        .expect_copy_metadata()
                        .withf(move |source_file, _| {
                            expected_source_files.contains(&PathBuf::from(source_file))
                        })
                        .returning(|_, _| Ok(()));

                    Ok(Box::new(metadata_writer))
                }
            });

        let result = convert_all_with_factory(
            utilities::get_paths_from_path_bufs(&source_files).as_slice(),
            destination_directory.path(),
            &element_factory,
        );

        assert_eq!(source_files.len(), result.unwrap().len());
    }

    #[test]
    fn destination_path_has_same_filename() {
        let source_directory = tempfile::TempDir::new().unwrap();
        let source_file = tempfile::Builder::new()
            .prefix("temp_file")
            .suffix(".wav")
            .tempfile_in(source_directory.path())
            .unwrap();

        let destination_directory = tempfile::TempDir::new().unwrap();

        let expected_destination_file = {
            let mut expected_filename = OsString::from(source_file.path().file_stem().unwrap());

            expected_filename.push(".mp3");

            let mut path = PathBuf::from(destination_directory.path());

            path.push(expected_filename);

            path
        };

        assert_eq!(
            expected_destination_file,
            get_output_mp3_path(source_file.path(), destination_directory.path()).unwrap()
        );
    }

    fn create_file(name: &str, source_directory: &tempfile::TempDir) -> PathBuf {
        let mut path = PathBuf::new();

        path.push(source_directory.path());
        path.push(name);

        File::create(&path).unwrap();

        path
    }

    #[test]
    fn convert_files_with_error() {
        let file_a = tempfile::NamedTempFile::new().unwrap();
        let file_b = tempfile::NamedTempFile::new().unwrap();
        let analyzed_file_a = tempfile::NamedTempFile::new().unwrap();
        let analyzed_file_b = tempfile::NamedTempFile::new().unwrap();

        let mut element_factory = MockElementFactory::new();
        let source_files = vec![PathBuf::from(file_a.path()), PathBuf::from(file_b.path())];
        let analyzed_source_files = vec![
            PathBuf::from(analyzed_file_a.path()),
            PathBuf::from(analyzed_file_b.path()),
        ];
        let destination_directory = tempfile::tempdir().unwrap();
        let error_analyzed_file_path = analyzed_source_files[1].clone();

        element_factory
            .expect_create_analyzer()
            .withf({
                let expected_source_files_for_factory = source_files.clone();

                move |source_files| source_files == expected_source_files_for_factory
            })
            .times(1)
            .returning({
                let expected_source_files_for_analyzer = source_files.clone();
                let analyzed_files = analyzed_source_files.clone();

                move |_| {
                    let mut analyzer = MockAnalyzer::new();
                    let expected_files = expected_source_files_for_analyzer.clone();
                    let analyzed_files = analyzed_files.clone();

                    analyzer
                        .expect_analyze()
                        .withf(move |source_files| source_files == expected_files)
                        .returning(move |_| Ok(analyzed_files.clone()));

                    Ok(Box::new(analyzer))
                }
            });

        element_factory
            .expect_create_mp3_converter()
            .withf({
                let expected_source_files = analyzed_source_files.clone();

                move |source_file| expected_source_files.contains(&PathBuf::from(source_file))
            })
            .times(source_files.len())
            .returning({
                let source_files_for_converter = analyzed_source_files.clone();

                move |path| {
                    if error_analyzed_file_path == path {
                        return Err(ConversionError::Unknown);
                    }

                    let mut mp3_converter = MockMp3Converter::new();
                    let expected_source_files = source_files_for_converter.clone();

                    mp3_converter
                        .expect_convert()
                        .withf(move |source_file, _| {
                            expected_source_files.contains(&PathBuf::from(source_file))
                        })
                        .returning(|_, _| Ok(()));

                    Ok(Box::new(mp3_converter))
                }
            });

        element_factory
            .expect_create_metadata_writer()
            .withf({
                let expected_source_files = source_files.clone();

                move |source_file| expected_source_files.contains(&PathBuf::from(source_file))
            })
            .times(1)
            .returning({
                let source_files_for_writer = source_files.clone();

                move |_| {
                    let mut metadata_writer = MockMetadataWriter::new();
                    let expected_source_files = source_files_for_writer.clone();

                    metadata_writer
                        .expect_copy_metadata()
                        .withf(move |source_file, _| {
                            expected_source_files.contains(&PathBuf::from(source_file))
                        })
                        .returning(|_, _| Ok(()));

                    Ok(Box::new(metadata_writer))
                }
            });

        let result = convert_all_with_factory(
            utilities::get_paths_from_path_bufs(&source_files).as_slice(),
            destination_directory.path(),
            &element_factory,
        );

        assert_eq!(
            ConversionError::Unknown.to_string(),
            result.err().unwrap().to_string()
        );
    }

    #[test]
    fn convert_files_with_same_name() {
        let source_directory = tempfile::tempdir().unwrap();

        let file_a = create_file("a.ogg", &source_directory);
        let file_b = create_file("a.wav", &source_directory);

        let source_files = vec![file_a.as_path(), file_b.as_path()];
        let destination_directory = tempfile::tempdir().unwrap();

        let result = convert_all_with_factory(
            &source_files,
            destination_directory.path(),
            &MockElementFactory::new(),
        );

        assert!(matches!(
            result,
            Err(ConversionError::DuplicatedFilename { filename }) if filename == "a",
        ));
    }

    #[test]
    fn convert_files_with_same_ignore_case_name() {
        let source_directory = tempfile::tempdir().unwrap();

        let file_a = create_file("a.ogg", &source_directory);
        let file_b = create_file("A.wav", &source_directory);

        let source_files = vec![file_a.as_path(), file_b.as_path()];
        let destination_directory = tempfile::tempdir().unwrap();

        let result = convert_all_with_factory(
            &source_files,
            destination_directory.path(),
            &MockElementFactory::new(),
        );

        assert!(matches!(
            result,
            Err(ConversionError::DuplicatedFilename { filename })
            if filename.to_ascii_lowercase() == "a",
        ));
    }

    #[test]
    fn convert_with_source_in_destination() {
        let duplicated_directory = tempfile::tempdir().unwrap();

        let file_a = create_file("a.ogg", &duplicated_directory);

        let source_files = vec![file_a.as_path()];

        let result = convert_all_with_factory(
            &source_files,
            duplicated_directory.path(),
            &MockElementFactory::new(),
        );

        assert!(matches!(
            result,
            Err(ConversionError::SourceFileInDestinationDirectory{source_file: source, destination_directory: directory})
            if source == file_a && directory == duplicated_directory.path(),
        ));
    }
}
