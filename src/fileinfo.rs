//! Access to the file-analyzing service
//! ([fileinfo](https://retdec.com/api/docs/fileinfo.html)).

use analysis::Analysis;
use analysis::AnalysisArguments;
use connection::APIArguments;
use connection::APIConnectionFactory;
use connection::HyperAPIConnectionFactory;
use connection::ResponseVerifyingAPIConnectionFactory;
use error::Result;
use error::ResultExt;
use settings::Settings;

/// File-analyzing service.
///
/// # Examples
///
/// ```no_run
/// use std::path::Path;
///
/// use retdec::analysis::AnalysisArguments;
/// use retdec::fileinfo::Fileinfo;
/// use retdec::settings::Settings;
///
/// let settings = Settings::new()
///     .with_api_key("MY-API-KEY");
/// let fileinfo = Fileinfo::new(settings);
/// let args = AnalysisArguments::new()
///     .with_input_file(Path::new(&"file.exe").to_path_buf());
/// let mut analysis = fileinfo.start_analysis(&args).unwrap();
/// analysis.wait_until_finished().unwrap();
/// let output = analysis.get_output().unwrap();
/// print!("{}", output);
/// ```
pub struct Fileinfo {
    conn_factory: Box<APIConnectionFactory>,
}

impl Fileinfo {
    /// Creates a new instance of the file-analyzing service.
    pub fn new(settings: Settings) -> Self {
        Fileinfo {
            conn_factory: Box::new(
                ResponseVerifyingAPIConnectionFactory::new(
                    Box::new(HyperAPIConnectionFactory::new(settings))
                )
            ),
        }
    }

    /// Starts a new file analysis with the given arguments.
    pub fn start_analysis(&self, args: &AnalysisArguments) -> Result<Analysis> {
        let mut conn = self.conn_factory.new_connection();
        let url = format!("{}/fileinfo/analyses", conn.api_url());
        let api_args = self.create_api_args(args)?;
        let response = conn.send_post_request(&url, &api_args)
            .chain_err(|| "failed to start an analysis")?;
        let id = response.json_value_as_string("id")
            .ok_or(format!("{} returned invalid JSON response", url))?;
        Ok(Analysis::new(id, conn))
    }

    fn create_api_args(&self, args: &AnalysisArguments) -> Result<APIArguments> {
        let mut api_args = APIArguments::new();
        api_args.add_opt_string_arg("output_format", args.output_format());
        api_args.add_opt_bool_arg("verbose", args.verbose());
        match args.input_file() {
            Some(ref input_file) => {
                api_args.add_file("input", input_file);
            }
            None => {
                bail!("no input file given");
            }
        }
        Ok(api_args)
    }

    #[cfg(test)]
    fn with_conn_factory(conn_factory: Box<APIConnectionFactory>) -> Self {
        Fileinfo { conn_factory: conn_factory }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::cell::RefCell;
    use std::path::Path;
    use std::rc::Rc;

    use analysis::AnalysisArguments;
    use connection::tests::APIArgumentsBuilder;
    use connection::tests::APIConnectionFactoryMock;
    use connection::tests::APIConnectionMock;
    use connection::tests::APIResponseBuilder;

    fn create_fileinfo(mut settings: Settings) -> (Rc<RefCell<APIConnectionMock>>, Fileinfo) {
        // Ensure that we always use the default API URL in tests (i.e. it is
        // not overridden when the RETDEC_API_URL environment variable is set).
        // Otherwise, APIConnectionMock.add_response() will not work properly.
        settings = settings.with_api_url("https://retdec.com/service/api");
        let conn = Rc::new(
            RefCell::new(
                APIConnectionMock::new(settings.clone())
            )
        );
        let conn_factory = Box::new(
            APIConnectionFactoryMock::new(
                settings.clone(),
                conn.clone()
            )
        );
        (conn, Fileinfo::with_conn_factory(conn_factory))
    }

    #[test]
    fn fileinfo_start_analysis_starts_analysis_with_correct_arguments() {
        let (conn, fileinfo) = create_fileinfo(Settings::new());
        let input_file = Path::new(&"file.exe").to_path_buf();
        let args = AnalysisArguments::new()
            .with_output_format("json")
            .with_verbose(true)
            .with_input_file(input_file.clone());
        conn.borrow_mut().add_response(
            "POST",
            "https://retdec.com/service/api/fileinfo/analyses",
            Ok(
                APIResponseBuilder::new()
                    .with_status_code(200)
                    .with_body(br#"{
                        "id": "ID"
                    }"#)
                    .build()
            )
        );

        let analysis = fileinfo.start_analysis(&args)
            .expect("analysis should have succeeded");

        assert_eq!(*analysis.id(), "ID");
        assert!(conn.borrow_mut().request_sent(
            "POST",
            "https://retdec.com/service/api/fileinfo/analyses",
            APIArgumentsBuilder::new()
                .with_string_arg("output_format", "json")
                .with_bool_arg("verbose", true)
                .with_file("input", input_file.clone())
                .build()
        ));
    }

    #[test]
    fn fileinfo_start_analysis_returns_error_when_input_file_is_not_given() {
        let (conn, fileinfo) = create_fileinfo(Settings::new());
        let args = AnalysisArguments::new();
        conn.borrow_mut().add_response(
            "POST",
            "https://retdec.com/service/api/fileinfo/analyses",
            Ok(
                APIResponseBuilder::new()
                    .with_status_code(200)
                    .with_body(br#"{
                        "id": "ID"
                    }"#)
                    .build()
            )
        );

        let result = fileinfo.start_analysis(&args);

        let err = result.err().expect("expected start_analysis() to fail");
        assert_eq!(err.description(), "no input file given");
    }

    #[test]
    fn fileinfo_start_analysis_returns_error_when_returned_json_does_not_contain_id() {
        let (conn, fileinfo) = create_fileinfo(Settings::new());
        let input_file = Path::new(&"file.exe").to_path_buf();
        let args = AnalysisArguments::new()
            .with_input_file(input_file.clone());
        conn.borrow_mut().add_response(
            "POST",
            "https://retdec.com/service/api/fileinfo/analyses",
            Ok(
                APIResponseBuilder::new()
                    .with_status_code(200)
                    .with_body(b"{}")
                    .build()
            )
        );

        let result = fileinfo.start_analysis(&args);

        let err = result.err().expect("expected start_analysis() to fail");
        assert_eq!(
            err.description(),
            "https://retdec.com/service/api/fileinfo/analyses returned invalid JSON response"
        );
    }
}
