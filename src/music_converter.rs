use std::{
    ffi::OsString,
    fs,
    path::{Path, PathBuf},
};

use crate::{
    conversion_error::ConversionError, element::ElementFactory, element::Elements, utilities,
};

pub fn convert_all(
    source_files_in_album: &[&Path],
    destination_directory: &Path,
) -> Result<Vec<PathBuf>, ConversionError> {
    let temporary_directory = utilities::create_temporary_directory()?;
    let elements = Elements::new(temporary_directory.path());

    convert_all_with_factory(source_files_in_album, destination_directory, &elements)
}

fn convert_all_with_factory(
    source_files_in_album: &[&Path],
    destination_directory: &Path,
    element_factory: &dyn ElementFactory,
) -> Result<Vec<PathBuf>, ConversionError> {
    let analyzed_files = analyze(source_files_in_album, element_factory)?;
    let mut mp3_file_paths = Vec::with_capacity(source_files_in_album.len());

    for (source_file, analyzed_file) in source_files_in_album.iter().zip(analyzed_files.iter()) {
        let result = convert_to_mp3_from(
            source_file,
            analyzed_file,
            destination_directory,
            element_factory,
        );

        match result {
            Ok(mp3_path) => mp3_file_paths.push(mp3_path),
            Err(error) => {
                remove_files(&mp3_file_paths);

                return Err(error);
            }
        }
    }

    Ok(mp3_file_paths)
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
    )?;

    Ok(output_mp3_path)
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
    use super::*;

    use std::{env, path::PathBuf, vec};

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
        let destination_directory = env::temp_dir();

        {
            let expected_source_files_for_factory = source_files.clone();
            let expected_source_files_for_analyzer = source_files.clone();
            let analyzed_files = analyzed_source_files.clone();

            element_factory
                .expect_create_analyzer()
                .withf(move |source_files| source_files == expected_source_files_for_factory)
                .times(1)
                .returning(move |_| {
                    let mut analyzer = MockAnalyzer::new();
                    let expected_files = expected_source_files_for_analyzer.clone();
                    let analyzed_files = analyzed_files.clone();

                    analyzer
                        .expect_analyze()
                        .withf(move |source_files| source_files == expected_files)
                        .returning(move |_| Ok(analyzed_files.clone()));

                    Ok(Box::new(analyzer))
                });
        }

        {
            let expected_source_files = analyzed_source_files.clone();
            let source_files_for_converter = analyzed_source_files.clone();

            element_factory
                .expect_create_mp3_converter()
                .withf(move |source_file| {
                    expected_source_files.contains(&PathBuf::from(source_file))
                })
                .times(source_files.len())
                .returning(move |_| {
                    let mut mp3_converter = MockMp3Converter::new();
                    let expected_source_files = source_files_for_converter.clone();

                    mp3_converter
                        .expect_convert()
                        .withf(move |source_file, _| {
                            expected_source_files.contains(&PathBuf::from(source_file))
                        })
                        .returning(|_, _| Ok(()));

                    Ok(Box::new(mp3_converter))
                });
        }

        {
            let expected_source_files = source_files.clone();
            let source_files_for_writer = source_files.clone();

            element_factory
                .expect_create_metadata_writer()
                .withf(move |source_file| {
                    expected_source_files.contains(&PathBuf::from(source_file))
                })
                .times(source_files.len())
                .returning(move |_| {
                    let mut metadata_writer = MockMetadataWriter::new();
                    let expected_source_files = source_files_for_writer.clone();

                    metadata_writer
                        .expect_copy_metadata()
                        .withf(move |source_file, _| {
                            expected_source_files.contains(&PathBuf::from(source_file))
                        })
                        .returning(|_, _| Ok(()));

                    Ok(Box::new(metadata_writer))
                });
        }

        let result = convert_all_with_factory(
            utilities::get_paths_from_path_bufs(&source_files).as_slice(),
            &destination_directory,
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
        let destination_directory = env::temp_dir();
        let error_analyzed_file_path = analyzed_source_files[1].clone();

        {
            let expected_source_files_for_factory = source_files.clone();
            let expected_source_files_for_analyzer = source_files.clone();
            let analyzed_files = analyzed_source_files.clone();

            element_factory
                .expect_create_analyzer()
                .withf(move |source_files| source_files == expected_source_files_for_factory)
                .times(1)
                .returning(move |_| {
                    let mut analyzer = MockAnalyzer::new();
                    let expected_files = expected_source_files_for_analyzer.clone();
                    let analyzed_files = analyzed_files.clone();

                    analyzer
                        .expect_analyze()
                        .withf(move |source_files| source_files == expected_files)
                        .returning(move |_| Ok(analyzed_files.clone()));

                    Ok(Box::new(analyzer))
                });
        }

        {
            let expected_source_files = analyzed_source_files.clone();
            let source_files_for_converter = analyzed_source_files.clone();

            element_factory
                .expect_create_mp3_converter()
                .withf(move |source_file| {
                    expected_source_files.contains(&PathBuf::from(source_file))
                })
                .times(source_files.len())
                .returning(move |path| {
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
                });
        }

        {
            let expected_source_files = source_files.clone();
            let source_files_for_writer = source_files.clone();

            element_factory
                .expect_create_metadata_writer()
                .withf(move |source_file| {
                    expected_source_files.contains(&PathBuf::from(source_file))
                })
                .times(1)
                .returning(move |_| {
                    let mut metadata_writer = MockMetadataWriter::new();
                    let expected_source_files = source_files_for_writer.clone();

                    metadata_writer
                        .expect_copy_metadata()
                        .withf(move |source_file, _| {
                            expected_source_files.contains(&PathBuf::from(source_file))
                        })
                        .returning(|_, _| Ok(()));

                    Ok(Box::new(metadata_writer))
                });
        }

        let result = convert_all_with_factory(
            utilities::get_paths_from_path_bufs(&source_files).as_slice(),
            &destination_directory,
            &element_factory,
        );

        assert_eq!(
            ConversionError::Unknown.to_string(),
            result.err().unwrap().to_string()
        );
    }
}
