mod common;

#[test]
fn analyze_single() {
    common::analyze_single("test1.ogg");
}

#[test]
fn analyze_album() {
    common::analyze_album(&["test1.ogg", "test2.ogg"]);
}

#[test]
fn convert() {
    common::convert_source("test1.ogg");
}

#[test]
fn copy_metadata() {
    common::copy_metadata("test1.ogg");
}