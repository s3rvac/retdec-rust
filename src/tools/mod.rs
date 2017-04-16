//! Tools that use the library to analyze and decompile files.

macro_rules! generate_main_for_tool {
    ($main:expr) => {
        /// Implementation of the `main()` function for the tool.
        ///
        /// Runs the tool. If the tool fails, it prints the error to the
        /// standard error. Then, it terminates the process. If the tool
        /// finished successfully, the exit code will be 0, otherwise 1.
        pub fn main() {
            if let Err(ref e) = $main(&::std::env::args().collect()) {
                ::error::print_error(e, &mut ::std::io::stderr());
                ::std::process::exit(1);
            }
        }
    }
}

pub mod decompiler;
pub mod fileinfo;
