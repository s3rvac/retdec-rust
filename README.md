# retdec-rust

[![docs.rs](https://docs.rs/retdec/badge.svg)](https://docs.rs/retdec)
[![crates.io](https://img.shields.io/crates/v/retdec.svg)](https://crates.io/crates/retdec)

## WARNING

The [retdec.com](https://retdec.com) decompilation service is to be disabled
(see the [official
announcement](https://retdec.com/news/?2018-06-07-New-Version-3-1)). This will
render the library and tools in the present repository non-functional. I will
keep the repository online in case it is helpful to anyone.

## Description

A Rust library and tools providing easy access to the
[retdec.com](https://retdec.com) decompilation service through their public
[REST API](https://retdec.com/api/).

You can either incorporate the library in your own tools:

```rust
extern crate retdec;

use retdec::{Decompiler, DecompilationArguments, File, Settings};

let decompiler = Decompiler::new(
    Settings::new()
        .with_api_key("YOUR-API-KEY")
);
let mut decompilation = decompiler.start_decompilation(
    DecompilationArguments::new()
        .with_input_file(File::from_path("hello.exe")?)
)?;
decompilation.wait_until_finished()?;
let output_code = decompilation.get_output_hll_code()?;
print!("{}", output_code);
```

or you can use the provided tool for stand-alone decompilations:

```text
$ decompiler -k YOUR-API-KEY hello.exe
```

Either way, you get the decompiled C code:

```text
//
// This file was generated by the Retargetable Decompiler
// Website: https://retdec.com
// Copyright (c) 2017 Retargetable Decompiler <info@retdec.com>
//

int main(int argc, char ** argv) {
    printf("Hello, world!\n");
    return 0;
}
```

Additionally, the crate provides access to the
[fileinfo](https://retdec.com/api/docs/fileinfo.html) service (analysis of
binary files).

## Status

Currently, the crate only provides very basic support for the
[decompilation](https://retdec.com/api/docs/decompiler.html) and
[file-analyzing](https://retdec.com/api/docs/fileinfo.html) services. **Support
for more features is under way as the crate is under development.**

A summary of all the currently supported parts of the [retdec.com's
API](https://retdec.com/api/docs/index.html) is available
[here](https://github.com/s3rvac/retdec-rust/tree/master/STATUS.md).

## Installation

To include the crate into your project so you can use it as a library, add the
following lines into your `Cargo.toml` file:

```
[dependencies]
retdec = "0.1.0"
```

If you want to use the development version (current `master` branch), use these
two lines instead:

```text
[dependencies]
retdec = { git = "https://github.com/s3rvac/retdec-rust" }
```

If you just want to use the command-line tools (`decompiler`, `fileinfo`),
install the project as follows:

```text
cargo install retdec
```

## Documentation

An automatically generated API documentation is available here:

* [master](https://projects.petrzemek.net/retdec-rust/doc/master/retdec/index.html)
  (development version)
* [0.1.0](https://docs.rs/retdec/0.1.0/retdec/) (latest stable version)

## Contributions

Contributions are welcome. Notes:

* To generate API documentation, run

    ```text
    cargo doc --lib --no-deps
    ```

* To run unit tests, execute

    ```text
    cargo test --lib
    ```

* To run documentation tests, execute

    ```text
    cargo test --doc
    ```

* To run all tests, including integration tests, execute

    ```text
    RETDEC_API_KEY=YOUR-API-KEY cargo test
    ```

  *Note*: Before running integration tests, you need to set the
  `RETDEC_API_KEY` environment variable to your API key. Integrations tests
  communicate with the `retdec.com`'s API, which is why a valid API key is
  needed.

## License

Licensed under either of

* Apache License, Version 2.0,
  ([LICENSE-APACHE](https://github.com/s3rvac/retdec-rust/tree/master/LICENSE-APACHE)
  or http://www.apache.org/licenses/LICENSE-2.0)
* MIT License
  ([LICENSE-MIT](https://github.com/s3rvac/retdec-rust/tree/master/LICENSE-APACHE)
  or http://opensource.org/licenses/MIT)

at your option.

## Access from Other Languages

If you want to access the [retdec.com](https://retdec.com) decompilation
service from other languages, check out the following projects:

* [retdec-python](https://github.com/s3rvac/retdec-python) - A library and
  tools for accessing the service from Python.
* [retdec-cpp](https://github.com/s3rvac/retdec-cpp) - A library and tools for
  accessing the service from C++.
* [retdec-sh](https://github.com/s3rvac/retdec-sh) - Scripts for accessing the
  service from shell.
