//! Integration tests for the `fileinfo` service and tool.
//!
//! Unit tests are in each module in the `src` directory.

extern crate json;
extern crate retdec;

mod common;

use common::path_to_sample;
use common::run_tool;

#[test]
fn fileinfo_correctly_analyzes_input_file() {
    let output = run_tool(
        "fileinfo", &[
            "--output-format", "json",
            "--verbose",
            &path_to_sample("pe-hello.exe")
        ]
    );

    // There should be no errors.
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(output.status.success(), "fileinfo failed; reason:\n{}", stderr);
    assert_eq!(stderr, "");
    let stdout = String::from_utf8_lossy(&output.stdout);
    // The output should be in the JSON format (--output-format json).
    let result = json::parse(&stdout)
        .expect(&format!("failed to parse the output as JSON:\n {}", stdout));
    let file_name = result["inputFile"].as_str()
        .expect("failed to get the name of the input file");
    assert_eq!(file_name, "pe-hello.exe");
    // There should be keys generated for verbose output (--verbose).
    assert!(result["sectionTable"].is_object());
    assert!(result["importTable"].is_object());
}
