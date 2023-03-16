mod common;

#[test]
fn analyze_single() {
    common::analyze_single("test1.mp3");
}

#[test]
fn analyze_album() {
    common::analyze_album(&["test1.mp3", "test2.mp3"]);
}

#[test]
fn convert() {
    common::convert_source("test1.mp3");
}

#[test]
fn copy_metadata() {
    common::copy_metadata("test1.mp3");
}
