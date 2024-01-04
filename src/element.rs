//! Managements of all elements.

mod aac;
mod common;
mod ffmpeg;
mod flac;
mod lame;
mod lofty;
mod mp3;
mod ogg_vorbis;
mod wav;

use std::path::{Path, PathBuf};

use crate::conversion_error::ConversionError;

use self::{
    aac::{AacGain, AacToMp3Converter},
    flac::{FlacToMp3Converter, MetaFlac},
    lofty::LoftyMetadataWriter,
    mp3::{Mp3Gain, Mp3Reconverter},
    ogg_vorbis::{OggVorbisGain, OggVorbisToMp3Converter},
    wav::WavFlacAnalyzer,
};

/// Analyzes music files.
#[cfg_attr(test, mockall::automock)]
#[allow(clippy::needless_lifetimes)]
pub trait Analyzer {
    /// Analyzes music files in an album.
    ///
    /// For example, replaygain is applied to music files.
    ///
    /// Music files of `source_paths_in_album` are analyzed. Analyzed music files are returned.
    fn analyze<'a>(
        &self,
        source_paths_in_album: &[&'a Path],
    ) -> Result<Vec<PathBuf>, ConversionError>;
}

/// Writer for the metadata of a music file.
#[cfg_attr(test, mockall::automock)]
pub trait MetadataWriter {
    /// Copies the metadata from the source file to the target file.
    fn copy_metadata(&self, source_file: &Path, target_file: &Path) -> Result<(), ConversionError>;
}

/// Converter that a music file to an MP3 file.
#[cfg_attr(test, mockall::automock)]
pub trait Mp3Converter {
    /// Converts a music file.
    ///
    /// `source_file` is converted to `destination_file` as MP3 file.
    fn convert(&self, source_file: &Path, destination_file: &Path) -> Result<(), ConversionError>;
}

/// Result about [`ElementFactory`].
pub type FactoryResult<E> = std::result::Result<Box<E>, ConversionError>;

/// Factory for elements.
#[cfg_attr(test, mockall::automock)]
#[allow(clippy::needless_lifetimes)]
pub trait ElementFactory {
    /// Creates an [`Analyzer`].
    ///
    /// An analyzer is created for source music files in an album.
    fn create_analyzer<'a>(
        &self,
        source_files_in_album: &[&'a Path],
    ) -> FactoryResult<dyn Analyzer>;

    /// Creates an [`Mp3Converter`].
    ///
    /// A converter is created for a source music file.
    fn create_mp3_converter(&self, source_file: &Path) -> FactoryResult<dyn Mp3Converter>;

    /// Creates a [`MetadataWriter`].
    ///
    /// A writer is created for a source music file.
    fn create_metadata_writer(&self, source_file: &Path) -> FactoryResult<dyn MetadataWriter>;
}

// non_exhaustive attribute works about crate.
// https://blog.rust-lang.org/2019/12/19/Rust-1.40.0.html#non_exhaustive-structs-enums-and-variants

struct ElementGenerator {
    is_file: fn(&Path) -> bool,
    create_analyzer: fn(&Path) -> Box<dyn Analyzer>,
    create_mp3_converter: fn() -> Box<dyn Mp3Converter>,
    create_metadata_writer: fn() -> Box<dyn MetadataWriter>,
}

/// Elements for music file conversion.
pub struct Elements {
    generators: Vec<ElementGenerator>,
    working_directory: PathBuf,
}

impl Elements {
    /// Creates [`Elements`] with a working directory.
    pub fn new(working_directory: &Path) -> Self {
        Elements {
            generators: vec![
                ElementGenerator {
                    is_file: |file| ogg_vorbis::is_ogg_vorbis(file),
                    create_analyzer: |_| Box::new(OggVorbisGain),
                    create_mp3_converter: || Box::new(OggVorbisToMp3Converter),
                    create_metadata_writer: || Box::new(LoftyMetadataWriter),
                },
                ElementGenerator {
                    is_file: |file| flac::is_flac(file),
                    create_analyzer: |_| Box::new(MetaFlac),
                    create_mp3_converter: || Box::new(FlacToMp3Converter),
                    create_metadata_writer: || Box::new(LoftyMetadataWriter),
                },
                ElementGenerator {
                    is_file: |file| mp3::is_mp3(file),
                    create_analyzer: |_| Box::new(Mp3Gain {}),
                    create_mp3_converter: || Box::new(Mp3Reconverter),
                    create_metadata_writer: || Box::new(LoftyMetadataWriter),
                },
                ElementGenerator {
                    is_file: |file| aac::is_aac(file),
                    create_analyzer: |_| Box::new(AacGain),
                    create_mp3_converter: || Box::new(AacToMp3Converter),
                    create_metadata_writer: || Box::new(LoftyMetadataWriter),
                },
                ElementGenerator {
                    is_file: |file| wav::is_wav(file),
                    create_analyzer: |working_directory| {
                        Box::new(WavFlacAnalyzer::new(working_directory))
                    },
                    // Mp3Converter is not used because WavFlacAnalyzer converts to FLAC.
                    create_mp3_converter: || Box::new(NotSupportedConverter),
                    create_metadata_writer: || Box::new(NullMetadataWriter),
                },
            ],
            working_directory: working_directory.to_path_buf(),
        }
    }
}

struct NullMetadataWriter;

impl MetadataWriter for NullMetadataWriter {
    fn copy_metadata(&self, _: &Path, _: &Path) -> Result<(), ConversionError> {
        Ok(())
    }
}

struct NotSupportedConverter;

impl Mp3Converter for NotSupportedConverter {
    fn convert(&self, _: &Path, _: &Path) -> Result<(), ConversionError> {
        Err(ConversionError::NotSupported)
    }
}

impl ElementFactory for Elements {
    fn create_analyzer(&self, source_files_in_album: &[&Path]) -> FactoryResult<dyn Analyzer> {
        self.generators
            .iter()
            .find(|generator| common::has_all_extension(source_files_in_album, generator.is_file))
            .map_or(Err(ConversionError::NotSupported), |generator| {
                Ok((generator.create_analyzer)(
                    self.working_directory.as_path(),
                ))
            })
    }

    fn create_mp3_converter(&self, source_file: &Path) -> FactoryResult<dyn Mp3Converter> {
        self.generators
            .iter()
            .find(|generator| (generator.is_file)(source_file))
            .map_or(Err(ConversionError::NotSupported), |generator| {
                Ok((generator.create_mp3_converter)())
            })
    }

    fn create_metadata_writer(&self, source_file: &Path) -> FactoryResult<dyn MetadataWriter> {
        self.generators
            .iter()
            .find(|generator| (generator.is_file)(source_file))
            .map_or(Err(ConversionError::NotSupported), |generator| {
                Ok((generator.create_metadata_writer)())
            })
    }
}
