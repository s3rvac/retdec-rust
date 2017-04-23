//! Tests for file representations that work with the filesystem.
//!
//! Unit tests are in each module in the `src` directory.

extern crate retdec;

#[allow(dead_code)]
mod common;

use common::path_to_sample;
use retdec::file::File;

#[test]
fn file_from_path_returns_correct_file() {
    let file = File::from_path(path_to_sample("pe-hello.exe"))
        .expect("failed to create a file for 'pe-hello.exe'");

    assert_eq!(file.content().len(), 75292);
    assert_eq!(file.name(), "pe-hello.exe");
}

#[test]
fn file_from_path_with_custom_name_returns_correct_file() {
    let path = path_to_sample("pe-hello.exe");
    let file = File::from_path_with_custom_name(path, "file.exe")
        .expect("failed to create a file for 'pe-hello.exe'");

    assert_eq!(file.content().len(), 75292);
    assert_eq!(file.name(), "file.exe");
}
