//! Tests for file representations that work with the filesystem.
//!
//! Unit tests are in each module in the `src` directory.

extern crate retdec;
extern crate tempdir;

#[allow(dead_code)]
mod common;

use tempdir::TempDir;

use common::path_to_sample;
use retdec::file::File;

#[test]
fn file_from_path_returns_correct_file() {
    let file = File::from_path(path_to_sample("pe-hello.exe"))
        .expect("failed to create a file for 'pe-hello.exe'");

    assert_eq!(file.content_len(), 75292);
    assert_eq!(file.name(), "pe-hello.exe");
}

#[test]
fn file_from_path_with_custom_name_returns_correct_file() {
    let path = path_to_sample("pe-hello.exe");
    let file = File::from_path_with_custom_name(path, "file.exe")
        .expect("failed to create a file for 'pe-hello.exe'");

    assert_eq!(file.content_len(), 75292);
    assert_eq!(file.name(), "file.exe");
}

#[test]
fn file_save_into_stores_file_into_given_directory() {
    let file = File::from_content_with_name(b"content", "file.txt");
    let tmp_dir = TempDir::new("retdec-file-test")
        .expect("failed to create a temporary directory");

    let file_path = file.save_into(tmp_dir.path())
        .expect("failed to save the file");

    let file = File::from_path(file_path)
        .expect("failed to read the stored file");
    assert_eq!(file.content(), b"content");
}

#[test]
fn file_save_into_under_name_stores_file_into_given_directory_under_given_name() {
    let file = File::from_content_with_name(b"content", "file.txt");
    let tmp_dir = TempDir::new("retdec-file-test")
        .expect("failed to create a temporary directory");

    let file_path = file.save_into_under_name(tmp_dir.path(), "new-file.txt")
        .expect("failed to save the file");

    let file = File::from_path(file_path)
        .expect("failed to read the stored file");
    assert_eq!(file.name(), "new-file.txt");
    assert_eq!(file.content(), b"content");
}

#[test]
fn file_save_as_stores_file_into_given_path() {
    let file = File::from_content_with_name(b"content", "file.txt");
    let tmp_dir = TempDir::new("retdec-file-test")
        .expect("failed to create a temporary directory");
    let file_path = tmp_dir.path().join("new-file.txt");

    file.save_as(&file_path)
        .expect("failed to save the file");

    let file = File::from_path(file_path)
        .expect("failed to read the stored file");
    assert_eq!(file.name(), "new-file.txt");
    assert_eq!(file.content(), b"content");
}
