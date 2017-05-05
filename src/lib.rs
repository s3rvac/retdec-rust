//! A Rust library and tools providing easy access to the
//! [retdec.com](https://retdec.com) decompilation service through their public
//! [REST API](https://retdec.com/api/).
//!
//! You can either incorporate the library in your own tools:
//!
//! ```no_run
//! use retdec::{Decompiler, DecompilationArguments, File, Settings};
//!
//! let decompiler = Decompiler::new(
//!     Settings::new()
//!         .with_api_key("YOUR-API-KEY")
//! );
//! let mut decompilation = decompiler.start_decompilation(
//!     DecompilationArguments::new()
//!         .with_input_file(File::from_path("hello.exe").unwrap())
//! ).unwrap();
//! decompilation.wait_until_finished().unwrap();
//! let output_code = decompilation.get_output_hll_code().unwrap();
//! print!("{}", output_code);
//! ```
//!
//! or you can use the provided tool for stand-alone decompilations:
//!
//! ```text
//! $ decompiler -k YOUR-API-KEY hello.exe
//! ```
//!
//! Either way, you get the decompiled C code:
//!
//! ```text
//! //
//! // This file was generated by the Retargetable Decompiler
//! // Website: https://retdec.com
//! // Copyright (c) 2017 Retargetable Decompiler <info@retdec.com>
//! //
//!
//! int main(int argc, char ** argv) {
//!     printf("Hello, world!\n");
//!     return 0;
//! }
//! ```
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
#![warn(missing_docs)]
#![warn(trivial_casts)]
#![warn(trivial_numeric_casts)]
#![warn(unused_extern_crates)]
#![warn(unused_import_braces)]
#![warn(unused_qualifications)]

extern crate clap;
extern crate hyper;
extern crate json;
extern crate regex;
extern crate multipart;
#[macro_use]
extern crate error_chain;

/// Crate version.
pub const VERSION: &'static str = env!("CARGO_PKG_VERSION");

pub mod analysis;
pub mod decompilation;
pub mod decompiler;
pub mod error;
pub mod file;
pub mod fileinfo;
pub mod settings;
pub mod tools;

// Reexports.
pub use analysis::Analysis;
pub use analysis::AnalysisArguments;
pub use decompilation::Decompilation;
pub use decompilation::DecompilationArguments;
pub use decompiler::Decompiler;
pub use error::Error;
pub use error::Result;
pub use file::File;
pub use fileinfo::Fileinfo;
pub use settings::Settings;

mod connection;
mod resource;
mod utils;
