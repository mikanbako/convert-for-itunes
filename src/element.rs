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
    wav::{WavFlacAnalyzer, WavFlacToMp3Converter},
};

pub type FactoryResult<E> = std::result::Result<Box<E>, ConversionError>;

#[cfg_attr(test, mockall::automock)]
pub trait Analyzer {
    fn analyze<'a>(
        &self,
        source_paths_in_album: &[&'a Path],
    ) -> Result<Vec<PathBuf>, ConversionError>;
}

#[cfg_attr(test, mockall::automock)]
pub trait MetadataWriter {
    fn copy_metadata(&self, source_file: &Path, target_file: &Path) -> Result<(), ConversionError>;
}

#[cfg_attr(test, mockall::automock)]
pub trait Mp3Converter {
    fn convert(&self, source_file: &Path, destination_file: &Path) -> Result<(), ConversionError>;
}

#[cfg_attr(test, mockall::automock)]
pub trait ElementFactory {
    fn create_analyzer<'a>(
        &self,
        source_files_in_album: &[&'a Path],
    ) -> FactoryResult<dyn Analyzer>;

    fn create_mp3_converter(&self, source_file: &Path) -> FactoryResult<dyn Mp3Converter>;

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

pub struct Elements {
    generators: Vec<ElementGenerator>,
    working_directory: PathBuf,
}

impl Elements {
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
                    create_analyzer: |_| Box::new(Mp3Gain::new()),
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
                    create_mp3_converter: || Box::new(WavFlacToMp3Converter),
                    create_metadata_writer: || Box::new(NullMetadataWriter),
                },
            ],
            working_directory: working_directory.to_owned(),
        }
    }
}

struct NullMetadataWriter;

impl MetadataWriter for NullMetadataWriter {
    fn copy_metadata(&self, _: &Path, _: &Path) -> Result<(), ConversionError> {
        Ok(())
    }
}

impl ElementFactory for Elements {
    fn create_analyzer(&self, source_files_in_album: &[&Path]) -> FactoryResult<dyn Analyzer> {
        for generator in self.generators.iter() {
            if common::has_all_extension(source_files_in_album, generator.is_file) {
                return Ok((generator.create_analyzer)(
                    self.working_directory.as_path(),
                ));
            }
        }

        Err(ConversionError::NotSupported)
    }

    fn create_mp3_converter(&self, source_file: &Path) -> FactoryResult<dyn Mp3Converter> {
        for generator in self.generators.iter() {
            if (generator.is_file)(source_file) {
                return Ok((generator.create_mp3_converter)());
            }
        }

        Err(ConversionError::NotSupported)
    }

    fn create_metadata_writer(&self, source_file: &Path) -> FactoryResult<dyn MetadataWriter> {
        for generator in self.generators.iter() {
            if (generator.is_file)(source_file) {
                return Ok((generator.create_metadata_writer)());
            }
        }

        Err(ConversionError::NotSupported)
    }
}
