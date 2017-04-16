//! A program for decompilation of binary files using the
//! [retdec.com](https://retdec.com) public [REST
//! API](https://retdec.com/api/). Internally, it uses the
//! [retdec-rust](https://github.com/s3rvac/retdec-rust) library.

extern crate retdec;

fn main() {
    retdec::tools::decompiler::main();
}
