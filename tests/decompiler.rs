//! Integration tests for the `decompiler` service and tool.
//!
//! Unit tests are in each module in the `src` directory.

extern crate json;
extern crate retdec;

#[allow(dead_code)]
mod common;

use common::path_to_sample;
use common::run_tool;

#[test]
fn fileinfo_correctly_decompiles_input_file() {
    let output = run_tool(
        "decompiler", &[
            &path_to_sample("pe-hello.exe")
        ]
    );

    // There should be no errors.
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(output.status.success(), "decompiler failed; reason:\n{}", stderr);
    assert_eq!(stderr, "");
    // The output should contain the decompiled C code.
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Hello, world!"));
}
