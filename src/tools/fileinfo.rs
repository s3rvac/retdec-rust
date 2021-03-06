//! A tool for analysis of binary files.

use std::io::Write;
use std::io;

use clap::App;
use clap::AppSettings;
use clap::Arg;
use clap::ArgMatches;

use VERSION;
use analysis::AnalysisArguments;
use error::Result;
use error::ResultExt;
use file::File;
use fileinfo::Fileinfo;
use settings::Settings;

fn parse_args<'a>(args: &[String]) -> ArgMatches<'a> {
    App::new("fileinfo")
        .version(VERSION)
        .about("Analyzes the given binary file via retdec.com's API.")
        .after_help("The output from the analysis is printed to the standard output.")
        .setting(AppSettings::ColorNever)
        .arg(Arg::with_name("FILE")
            .required(true)
            .help("Input binary file to be analyzed"))
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
        .arg(Arg::with_name("output_format")
            .short("f")
            .long("output-format")
            .takes_value(true)
            .value_name("FORMAT")
            .possible_values(&["plain", "json"])
            .default_value("plain")
            .help("Format of the output from the analysis"))
        .arg(Arg::with_name("verbose")
            .short("v")
            .long("verbose")
            .help("Print all available information about the input file"))
        .get_matches_from(args)
}

fn print_analysis_result(output: &str) -> Result<()> {
    let mut stdout = io::stdout();
    stdout.write(output.as_bytes())
        .chain_err(|| "failed to print the result on the standard output")?;
    Ok(())
}

fn run(args: &[String]) -> Result<()> {
    let args = parse_args(args);

    let mut settings = Settings::new();
    if let Some(api_key) = args.value_of("api_key") {
        settings.set_api_key(api_key);
    }
    if let Some(api_url) = args.value_of("api_url") {
        settings.set_api_url(api_url);
    }
    let input_file = args.value_of("FILE")
        .expect("no FILE - this should never happen");

    let fileinfo = Fileinfo::new(settings);
    let args = AnalysisArguments::new()
        .with_output_format(
            args.value_of("output_format")
                .expect("no output_format - this should never happen")
        )
        .with_verbose(args.is_present("verbose"))
        .with_input_file(File::from_path(&input_file)?);
    let mut analysis = fileinfo.start_analysis(args)?;
    analysis.wait_until_finished()?;
    let output = analysis.get_output()?;
    print_analysis_result(&output)?;
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
                args.push("fileinfo".to_string());
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

    #[test]
    fn parse_args_sets_plain_as_default_output_format() {
        let args = parse_args(&args!["file.exe"]);
        assert_eq!(args.value_of("output_format"), Some("plain"));
    }

    #[test]
    fn parse_args_correctly_parses_output_format_short_form() {
        let args = parse_args(&args!["-f", "json", "file.exe"]);
        assert_eq!(args.value_of("output_format"), Some("json"));
    }

    #[test]
    fn parse_args_correctly_parses_output_format_long_form() {
        let args = parse_args(&args!["--output-format", "json", "file.exe"]);
        assert_eq!(args.value_of("output_format"), Some("json"));
    }

    #[test]
    fn parse_args_does_not_set_verbose_by_default() {
        let args = parse_args(&args!["file.exe"]);
        assert!(!args.is_present("verbose"));
    }

    #[test]
    fn parse_args_correctly_parses_verbose_short_form() {
        let args = parse_args(&args!["-v", "file.exe"]);
        assert!(args.is_present("verbose"));
    }

    #[test]
    fn parse_args_correctly_parses_verbose_long_form() {
        let args = parse_args(&args!["--verbose", "file.exe"]);
        assert!(args.is_present("verbose"));
    }
}
