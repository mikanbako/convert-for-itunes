mod common;

#[test]
fn analyze_single() {
    common::analyze_single("test1.m4a");
}

#[test]
fn analyze_album() {
    common::analyze_album(&["test1.m4a", "test2.m4a"]);
}

#[test]
fn convert() {
    common::convert_source("test1.m4a");
}

#[test]
fn copy_metadata() {
    common::copy_metadata("test1.m4a");
}
