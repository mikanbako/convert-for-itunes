// SPDX-FileCopyrightText: 2024 Keita Kita <maoutwo@gmail.com>
//
// SPDX-License-Identifier: MIT

mod common;

#[test]
fn analyze_single() {
    common::analyze_single("test1.flac");
}

#[test]
fn analyze_album() {
    common::analyze_album(&["test1.flac", "test2.flac"]);
}

#[test]
fn convert() {
    common::convert_source("test1.flac");
}

#[test]
fn copy_metadata() {
    common::copy_metadata("test1.flac");
}
