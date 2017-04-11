//! A tool for analysis of binary files.

use std::env;
use std::path::Path;
use std::process;

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

generate_main_for_tool!(run);
