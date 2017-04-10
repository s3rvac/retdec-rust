//! A tool for analysis of binary files.

use std::env;
use std::io;
use std::path::Path;
use std::process;

use error::Result;
use analysis::AnalysisArguments;
use error::Result;
use fileinfo::Fileinfo;
use settings::Settings;

fn run() -> Result<()> {
    let args: Vec<String> = env::args().skip(1).collect();
    if args.len() != 3 {
        println!("usage: fileinfo API_KEY API_URL FILE");
        process::exit(0);
    }

    let fileinfo = Fileinfo::new(
        Settings::new()
            .with_api_key(&args[0])
            .with_api_url(&args[1])
    );
    let args = AnalysisArguments::new()
        .with_input_file(Path::new(&args[2]).to_path_buf());
    let mut analysis = fileinfo.start_analysis(args)?;
    analysis.wait_until_finished()?;
    let output = analysis.get_output()?;
    print!("{}", output);
    Ok(())
}

/// Implementation of the `main()` function for the tool.
///
/// Runs the tool. If the tool fails, it prints the error to the standard
/// error. Then, it terminates the process. If the tool finished successfully,
/// the exit code will be 0, otherwise 1.
pub fn main() {
    if let Err(ref e) = run() {
        print_error(e, &mut io::stderr());
        process::exit(1);
    }
}
