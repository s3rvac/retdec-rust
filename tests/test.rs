//! Tests for access to the testing service.
//!
//! Unit tests are in each module in the `src` directory.

extern crate retdec;

#[allow(dead_code)]
mod common;

use std::collections::HashMap;

use retdec::settings::Settings;
use retdec::test::Test;
use common::ensure_api_key_is_set;

#[test]
fn auth_succeeds() {
    ensure_api_key_is_set();
    let settings = Settings::new();
    let test = Test::new(settings);

    let result = test.auth();

    assert!(result.is_ok());
}

#[test]
fn echo_returns_back_input_parameters() {
    ensure_api_key_is_set();
    let settings = Settings::new();
    let test = Test::new(settings);
    let mut params = HashMap::new();
    params.insert("param1".to_string(), "value1".to_string());
    params.insert("param2".to_string(), "value2".to_string());

    let result = test.echo(&params)
        .expect("expected echo() to succeed");

    assert_eq!(result.get("param1"), Some(&"value1".to_string()));
    assert_eq!(result.get("param2"), Some(&"value2".to_string()));
}
