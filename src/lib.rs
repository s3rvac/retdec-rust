//! A Rust library providing easy access to the
//! [retdec.com](https://retdec.com) decompilation service through their public
//! [REST API](https://retdec.com/api/).
//!
//! # Status
//!
//! The library is **at the beginning of its development** and its state is
//! **pre-alpha** (**highly experimental**).
//!
//! # License
//!
//! Licensed under either of
//!
//! * Apache License, Version 2.0,
//!   ([LICENSE-APACHE](https://github.com/s3rvac/retdec-rust/tree/master/LICENSE-APACHE)
//!   or http://www.apache.org/licenses/LICENSE-2.0)
//! * MIT License
//!   ([LICENSE-MIT](https://github.com/s3rvac/retdec-rust/tree/master/LICENSE-APACHE)
//!   or http://opensource.org/licenses/MIT)
//!
//! at your option.

// `error_chain!` can recurse deeply.
#![recursion_limit = "1024"]

// Add more lint checks.
#![deny(unsafe_code)]
#![deny(unstable_features)]
#![warn(trivial_casts)]
#![warn(trivial_numeric_casts)]
#![warn(unused_qualifications)]
#![warn(unused_extern_crates)]
#![warn(unused_import_braces)]

extern crate clap;
extern crate hyper;
extern crate json;
extern crate multipart;
#[macro_use]
extern crate error_chain;

/// Crate version.
pub const VERSION: &'static str = env!("CARGO_PKG_VERSION");

pub mod analysis;
pub mod connection;
pub mod error;
pub mod fileinfo;
pub mod settings;
pub mod tools;
