//! Common functionality for integration tests.

use std::env;
use std::path::PathBuf;
use std::process::Command;
use std::process::Output;

pub fn run_tool(tool: &str, args: &[&str]) -> Output {
    ensure_api_key_is_set();
    Command::new("cargo")
        .args(&["run", "--quiet", "--bin", tool, "--"])
        .args(args)
        .output()
        .expect("failed to execute the command")
}

pub fn path_to_sample(name: &str) -> String {
    let cwd = env::current_dir()
        .expect("failed to get the current working directory");
    let mut path = PathBuf::from(cwd);
    path.push("tests");
    path.push("samples");
    path.push(name);
    path.as_path().to_str()
        .expect("failed to convert the path into a string")
        .to_string()
}

fn ensure_api_key_is_set() {
    env::var("RETDEC_API_KEY")
        .expect("RETDEC_API_KEY has to be set to run this test");
}
