//! A tool for decompilation of files.

use std::io::Write;
use std::io;

use clap::App;
use clap::AppSettings;
use clap::Arg;
use clap::ArgMatches;

use VERSION;
use decompilation::DecompilationArguments;
use decompiler::Decompiler;
use error::Result;
use error::ResultExt;
use file::File;
use settings::Settings;

fn parse_args<'a>(args: &Vec<String>) -> ArgMatches<'a> {
    App::new("decompiler")
        .version(VERSION)
        .about("Decompiles the given file via retdec.com's API.")
        .setting(AppSettings::ColorNever)
        .arg(Arg::with_name("FILE")
            .required(true)
            .help("Input file to be decompiled"))
        .arg(Arg::with_name("api_key")
            .short("k")
            .long("api-key")
            .takes_value(true)
            .value_name("KEY")
            // It is important not to require the API key by default because it
            // enables the use of the RETDEC_API_KEY environment variable.
            .help("API key to be used."))
        .arg(Arg::with_name("api_url")
            .short("u")
            .long("api-url")
            .takes_value(true)
            .value_name("URL")
            // It is important not to require the API URL by default because it
            // enables the use of the RETDEC_API_URL environment variable.
            .help("Custom URL to the retdec.com's API."))
        .get_matches_from(args)
}

fn print_decompilation_result(output: &str) -> Result<()> {
    let mut stdout = io::stdout();
    stdout.write(output.as_bytes())
        .chain_err(|| "failed to print the result on the standard output")?;
    Ok(())
}

fn run(args: &Vec<String>) -> Result<()> {
    let args = parse_args(args);

    let mut settings = Settings::new();
    if let Some(api_key) = args.value_of("api_key") {
        settings = settings.with_api_key(api_key);
    }
    if let Some(api_url) = args.value_of("api_url") {
        settings = settings.with_api_url(api_url);
    }
    let input_file = args.value_of("FILE").unwrap();

    let decompiler = Decompiler::new(settings);
    let args = DecompilationArguments::new()
        .with_input_file(File::from_path(&input_file)?);
    let mut decompilation = decompiler.start_decompilation(args)?;
    decompilation.wait_until_finished()?;
    let output_code = decompilation.get_output_hll_code()?;
    print_decompilation_result(&output_code)?;
    Ok(())
}

generate_main_for_tool!(run);

#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! args {
        ($($arg:expr),*) => {
            {
                let mut args = Vec::new();
                args.push("decompiler".to_string());
                $(
                    args.push($arg.to_string());
                )*
                args
            }
        }
    }

    #[test]
    fn parse_args_correctly_parses_input_file() {
        let args = parse_args(&args!["file.exe"]);
        assert_eq!(args.value_of("FILE"), Some("file.exe"));
    }

    #[test]
    fn parse_args_correctly_parses_api_key_short_form() {
        let args = parse_args(&args!["-k", "KEY", "file.exe"]);
        assert_eq!(args.value_of("api_key"), Some("KEY"));
    }

    #[test]
    fn parse_args_correctly_parses_api_key_long_form() {
        let args = parse_args(&args!["--api-key", "KEY", "file.exe"]);
        assert_eq!(args.value_of("api_key"), Some("KEY"));
    }

    #[test]
    fn parse_args_correctly_parses_api_url_short_form() {
        let args = parse_args(&args!["-u", "URL", "file.exe"]);
        assert_eq!(args.value_of("api_url"), Some("URL"));
    }

    #[test]
    fn parse_args_correctly_parses_api_url_long_form() {
        let args = parse_args(&args!["--api-url", "URL", "file.exe"]);
        assert_eq!(args.value_of("api_url"), Some("URL"));
    }
}
