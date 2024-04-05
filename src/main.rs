// SPDX-FileCopyrightText: 2024 Keita Kita <maoutwo@gmail.com>
//
// SPDX-License-Identifier: MIT

use std::process::exit;

use clap::Parser;

use convert_for_itunes::convert_for_itunes::{convert_for_itunes, ConvertForITunesError, Setting};
use env_logger::Env;
use log::error;

fn initialize_logging() {
    env_logger::Builder::from_env(Env::default().default_filter_or("info"))
        .format_target(false)
        .format_timestamp(None)
        .init();
}

fn main() {
    initialize_logging();

    let result = convert_for_itunes(&Setting::parse());

    if result.is_err() {
        match result.unwrap_err() {
            ConvertForITunesError::ConversionError(error) => {
                error!("Conversion is failed. Detail: {error}");
            }
            ConvertForITunesError::DirectoryCannotBeCreated(directory, error) => {
                error!("{directory:?} cannot be created. Detail: {error}",);
            }
            ConvertForITunesError::MovingSourceFileIsFailed(error) => {
                error!("Moving files is failed. Detail: {error}");
            }
        }

        exit(1);
    }
}
