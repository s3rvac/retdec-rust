# retdec-rust

A Rust library and tools providing easy access to the
[retdec.com](https://retdec.com) decompilation service through their public
[REST API](https://retdec.com/api/).

## Status

The library is **at the beginning of its development** and its state is
**pre-alpha** (**highly experimental**).

## Installation

Currently, the only way of using the library and tools is to specify the
dependency from this git repository. To do that, add the following lines into
your `Cargo.toml` file:

```text
[dependencies]
retdec = { git = "https://github.com/s3rvac/retdec-rust" }
```

As soon as the first version is released, you will be able to add a dependency
directly from [crates.io](https://crates.io/).

## Documentation

An automatically generated API documentation will be available after the first
version is released.

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

* [retdec-cpp](https://github.com/s3rvac/retdec-cpp) - A library and tools for
  accessing the service from C++.
* [retdec-python](https://github.com/s3rvac/retdec-python) - A library and
  tools for accessing the service from Python.
* [retdec-sh](https://github.com/s3rvac/retdec-sh) - Scripts for accessing the
  service from shell.
