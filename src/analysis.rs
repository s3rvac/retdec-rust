//! Analyses from the fileinfo service.

use std::time::Duration;

use connection::APIConnection;
use connection::APIResponse;
use error::Result;
use error::ResultExt;
use file::File;
use resource::Resource;

/// Arguments for a file analysis.
///
/// # Examples
///
/// ```no_run
/// # use retdec::error::Result;
/// # fn test() -> Result<()> {
/// use retdec::analysis::AnalysisArguments;
/// use retdec::file::File;
///
/// let args = AnalysisArguments::new()
///     .with_output_format("json")
///     .with_verbose(true)
///     .with_input_file(File::from_path("file.exe")?);
/// # Ok(()) } fn main() { test().unwrap() }
/// ```
#[derive(Clone, Debug, Default)]
pub struct AnalysisArguments {
    output_format: Option<String>,
    verbose: Option<bool>,
    input_file: Option<File>,
}

impl AnalysisArguments {
    /// Returns new arguments initialized to default values.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the format of the output from the analysis.
    ///
    /// Available values are: `plain` (default), `json`.
    pub fn with_output_format<F>(mut self, output_format: F) -> Self
        where F: Into<String>
    {
        self.set_output_format(output_format);
        self
    }

    /// Should the analysis return all available information about the file?
    ///
    /// By default, the analysis returns only an abridged version.
    pub fn with_verbose(mut self, verbose: bool) -> Self {
        self.set_verbose(verbose);
        self
    }

    /// Sets the file to be analyzed.
    ///
    /// This parameter is required. Without it, there is nothing to analyze.
    pub fn with_input_file(mut self, input_file: File) -> Self {
        self.set_input_file(input_file);
        self
    }

    /// Sets the format of the output from the analysis.
    ///
    /// Available values are: `plain` (default), `json`.
    pub fn set_output_format<F>(&mut self, output_format: F)
        where F: Into<String>
    {
        self.output_format = Some(output_format.into());
    }

    /// Should the analysis return all available information about the file?
    ///
    /// By default, the analysis returns only an abridged version.
    pub fn set_verbose(&mut self, verbose: bool) {
        self.verbose = Some(verbose);
    }

    /// Sets the file to be analyzed.
    ///
    /// This parameter is required. Without it, there is nothing to analyze.
    pub fn set_input_file(&mut self, input_file: File) {
        self.input_file = Some(input_file);
    }

    /// Returns the output format.
    pub fn output_format(&self) -> Option<&String> {
        self.output_format.as_ref()
    }

    /// Returns whether the analysis should return all available information
    /// about the file.
    pub fn verbose(&self) -> Option<bool> {
        self.verbose
    }

    /// Returns the the file to be analyzed.
    pub fn input_file(&self) -> Option<&File> {
        self.input_file.as_ref()
    }

    /// Takes ownership of the output format and sets it to `None`.
    pub fn take_output_format(&mut self) -> Option<String> {
        self.output_format.take()
    }

    /// Takes ownership of the input file and sets it to `None`.
    pub fn take_input_file(&mut self) -> Option<File> {
        self.input_file.take()
    }
}

/// Analysis from the fileinfo service.
pub struct Analysis {
    resource: Resource,
}

impl Analysis {
    /// Creates access to an analysis with the given ID.
    ///
    /// Only for internal use.
    #[doc(hidden)]
    pub fn new<I: Into<String>>(id: I, conn: Box<APIConnection>) -> Self {
        Analysis {
            resource: Resource::new("fileinfo", "analyses", id, conn)
        }
    }

    /// Returns the ID of the analysis.
    ///
    /// Does not access the API.
    pub fn id(&self) -> &String {
        &self.resource.id
    }

    /// Has the analysis finished?
    ///
    /// Does not access the API, so the returned value may be outdated. In a
    /// greater detail, when it returns `true`, the analysis has surely
    /// finished. However, when it returns `false`, the analysis might or might
    /// not have finished. If you want to have an up-to-date information, use
    /// `has_finished()` instead.
    pub fn finished(&self) -> bool {
        self.resource.finished
    }

    /// Has the analysis finished?
    ///
    /// Accesses the API.
    pub fn has_finished(&mut self) -> Result<bool> {
        self.resource.has_finished()
    }

    /// Has the analysis succeeded?
    ///
    /// Does not access the API, so the returned value may be outdated. If you
    /// want to have an up-to-date information, use `has_succeeded()` instead.
    ///
    /// The returned value makes sense only when the analysis has finished.
    pub fn succeeded(&self) -> bool {
        self.resource.finished
    }

    /// Has the analysis succeeded?
    ///
    /// Accesses the API.
    ///
    /// The returned value makes sense only when the analysis has finished.
    pub fn has_succeeded(&mut self) -> Result<bool> {
        self.resource.has_succeeded()
    }

    /// Has the analysis failed?
    ///
    /// Does not access the API, so the returned value may be outdated. If you
    /// want to have an up-to-date information, use `has_failed()` instead.
    ///
    /// The returned value makes sense only when the analysis has finished.
    pub fn failed(&self) -> bool {
        self.resource.finished
    }

    /// Has the analysis failed?
    ///
    /// Accesses the API.
    ///
    /// The returned value makes sense only when the analysis has finished.
    pub fn has_failed(&mut self) -> Result<bool> {
        self.resource.has_failed()
    }

    /// Returns the error message (if any).
    ///
    /// Does not access the API, so the returned value may be outdated. If you
    /// want to have an up-to-date information, use `get_error()` instead.
    ///
    /// Calling this method makes sense only when the analysis has failed.
    /// Otherwise, it will always return `None`.
    pub fn error(&self) -> Option<&str> {
        self.resource.error()
    }

    /// Returns the error message (if any).
    ///
    /// Accesses the API.
    ///
    /// Calling this method makes sense only when the analysis has failed.
    /// Otherwise, it will always return `None`.
    pub fn get_error(&mut self) -> Result<Option<&str>> {
        self.resource.get_error()
    }

    /// Waits until the analysis has finished.
    ///
    /// When this method returns `Ok()`, the analysis has finished.
    ///
    /// Accesses the API.
    pub fn wait_until_finished(&mut self) -> Result<()> {
        // Currently, the retdec.com's API does not support push notifications,
        // so we have to poll for the status ourselves.
        while !self.finished() {
            self.resource.wait_for(Duration::from_millis(500));

            self.resource.update_status()
                .chain_err(|| "failed to update analysis status")?;
            if self.finished() {
                break;
            }
        }
        Ok(())
    }

    /// Returns the output from the analysis.
    ///
    /// The format of the output depends on the format selected when starting
    /// an analysis (`output_format`).
    ///
    /// Accesses the API.
    pub fn get_output(&mut self) -> Result<String> {
        let response = self.get_output_response()?;
        response.body_as_string()
    }

    /// Returns the output from the analysis as a file.
    ///
    /// The format of the output depends on the format selected when starting
    /// an analysis (`output_format`).
    ///
    /// Accesses the API.
    pub fn get_output_as_file(&mut self) -> Result<File> {
        let response = self.get_output_response()?;
        response.body_as_file()
    }

    fn get_output_response(&mut self) -> Result<APIResponse> {
        self.ensure_analysis_has_succeeded()?;
        let output_url = format!("{}/output", self.resource.base_url);
        self.resource.conn.send_get_request_without_args(&output_url)
    }

    fn ensure_analysis_has_succeeded(&mut self) -> Result<()> {
        self.resource.ensure_has_succeeded("analysis")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::cell::RefCell;
    use std::rc::Rc;

    use connection::tests::APIArgumentsBuilder;
    use connection::tests::APIConnectionMock;
    use connection::tests::APIConnectionMockWrapper;
    use connection::tests::APIResponseBuilder;
    use settings::Settings;

    fn create_analysis() -> (Rc<RefCell<APIConnectionMock>>, Analysis) {
        // We need to force an API URL to prevent it from being overridden by
        // setting the RETDEC_API_URL environment variable.
        let settings = Settings::new()
            .with_api_key("test")
            .with_api_url("https://retdec.com/service/api");
        let conn = Rc::new(RefCell::new(APIConnectionMock::new(settings.clone())));
        let conn_wrapper = Box::new(APIConnectionMockWrapper::new(conn.clone()));
        (conn, Analysis::new("ID", conn_wrapper))
    }

    fn make_analysis_succeed(conn: &Rc<RefCell<APIConnectionMock>>,
                             analysis: &mut Analysis) {
        conn.borrow_mut().add_response(
            "GET",
            "https://retdec.com/service/api/fileinfo/analyses/ID/status",
            Ok(
                APIResponseBuilder::new()
                    .with_status_code(200)
                    .with_body(br#"{
                        "finished": true,
                        "succeeded": true,
                        "failed": false
                    }"#)
                    .build()
            )
        );
        analysis.wait_until_finished()
            .expect("expected the analysis to finish successfully");
        assert!(conn.borrow().request_sent(
            "GET",
            "https://retdec.com/service/api/fileinfo/analyses/ID/status",
            APIArgumentsBuilder::new()
                .build()
        ));
        conn.borrow_mut().reset();
    }

    fn make_analysis_fail(conn: &Rc<RefCell<APIConnectionMock>>,
                          analysis: &mut Analysis,
                          error: &str) {
        conn.borrow_mut().add_response(
            "GET",
            "https://retdec.com/service/api/fileinfo/analyses/ID/status",
            Ok(
                APIResponseBuilder::new()
                    .with_status_code(200)
                    .with_body((r#"{
                        "finished": true,
                        "succeeded": false,
                        "failed": true,
                        "error": ""#.to_owned() + error + "\"}").as_bytes())
                    .build()
            )
        );
        analysis.wait_until_finished()
            .expect("expected the analysis to finish successfully");
        assert!(conn.borrow().request_sent(
            "GET",
            "https://retdec.com/service/api/fileinfo/analyses/ID/status",
            APIArgumentsBuilder::new()
                .build()
        ));
        conn.borrow_mut().reset();
    }

    #[test]
    fn analysis_id_returns_id_of_analysis() {
        let (_, analysis) = create_analysis();

        assert_eq!(analysis.id(), "ID");
    }

    #[test]
    fn analysis_finished_returns_false_when_analysis_has_not_finished() {
        let (_, analysis) = create_analysis();

        assert!(!analysis.finished());
    }

    #[test]
    fn analysis_finished_returns_true_when_analysis_has_finished() {
        let (conn, mut analysis) = create_analysis();
        make_analysis_succeed(&conn, &mut analysis);

        assert!(analysis.finished());
    }

    #[test]
    fn analysis_has_finished_returns_true_and_does_not_update_status_when_analysis_has_finished() {
        let (conn, mut analysis) = create_analysis();
        make_analysis_succeed(&conn, &mut analysis);

        let finished = analysis.has_finished()
            .expect("has_finished() should have succeeded");

        assert!(finished);
        assert!(conn.borrow().no_requests_sent());
    }

    #[test]
    fn analysis_has_finished_checks_status_when_analysis_has_not_yet_finished() {
        let (conn, mut analysis) = create_analysis();
        conn.borrow_mut().add_response(
            "GET",
            "https://retdec.com/service/api/fileinfo/analyses/ID/status",
            Ok(
                APIResponseBuilder::new()
                    .with_status_code(200)
                    .with_body(br#"{
                        "finished": true,
                        "succeeded": true,
                        "failed": false
                    }"#)
                    .build()
            )
        );

        let finished = analysis.has_finished()
            .expect("has_finished() should have succeeded");

        assert!(finished);
        assert!(conn.borrow().request_sent(
            "GET",
            "https://retdec.com/service/api/fileinfo/analyses/ID/status",
            APIArgumentsBuilder::new()
                .build()
        ));
    }

    #[test]
    fn analysis_succeeded_returns_false_when_analysis_has_not_yet_finished() {
        let (_, analysis) = create_analysis();

        assert!(!analysis.succeeded());
    }

    #[test]
    fn analysis_succeeded_returns_true_when_analysis_has_succeeded() {
        let (conn, mut analysis) = create_analysis();
        make_analysis_succeed(&conn, &mut analysis);

        assert!(analysis.succeeded());
    }

    #[test]
    fn analysis_has_succeeded_returns_true_and_does_not_update_status_when_analysis_has_succeeded() {
        let (conn, mut analysis) = create_analysis();
        make_analysis_succeed(&conn, &mut analysis);

        let succeeded = analysis.has_succeeded()
            .expect("has_succeeded() should have succeeded");

        assert!(succeeded);
        assert!(conn.borrow().no_requests_sent());
    }

    #[test]
    fn analysis_has_succeeded_checks_status_when_analysis_has_not_yet_succeeded() {
        let (conn, mut analysis) = create_analysis();
        conn.borrow_mut().add_response(
            "GET",
            "https://retdec.com/service/api/fileinfo/analyses/ID/status",
            Ok(
                APIResponseBuilder::new()
                    .with_status_code(200)
                    .with_body(br#"{
                        "finished": true,
                        "succeeded": true,
                        "failed": false
                    }"#)
                    .build()
            )
        );

        let succeeded = analysis.has_succeeded()
            .expect("has_succeeded() should have succeeded");

        assert!(succeeded);
        assert!(conn.borrow().request_sent(
            "GET",
            "https://retdec.com/service/api/fileinfo/analyses/ID/status",
            APIArgumentsBuilder::new()
                .build()
        ));
    }

    #[test]
    fn analysis_failed_returns_false_when_analysis_has_not_yet_finished() {
        let (_, analysis) = create_analysis();

        assert!(!analysis.failed());
    }

    #[test]
    fn analysis_failed_returns_true_when_analysis_has_failed() {
        let (conn, mut analysis) = create_analysis();
        make_analysis_fail(&conn, &mut analysis, "unknown error");

        assert!(analysis.failed());
    }

    #[test]
    fn analysis_has_failed_returns_true_and_does_not_update_status_when_analysis_has_failed() {
        let (conn, mut analysis) = create_analysis();
        make_analysis_fail(&conn, &mut analysis, "unknown error");

        let failed = analysis.has_failed()
            .expect("has_failed() should have succeeded");

        assert!(failed);
        assert!(conn.borrow().no_requests_sent());
    }

    #[test]
    fn analysis_has_failed_checks_status_when_analysis_has_not_yet_failed() {
        let (conn, mut analysis) = create_analysis();
        conn.borrow_mut().add_response(
            "GET",
            "https://retdec.com/service/api/fileinfo/analyses/ID/status",
            Ok(
                APIResponseBuilder::new()
                    .with_status_code(200)
                    .with_body(br#"{
                        "finished": true,
                        "succeeded": false,
                        "failed": true
                    }"#)
                    .build()
            )
        );

        let failed = analysis.has_failed()
            .expect("has_failed() should have succeeded");

        assert!(failed);
        assert!(conn.borrow().request_sent(
            "GET",
            "https://retdec.com/service/api/fileinfo/analyses/ID/status",
            APIArgumentsBuilder::new()
                .build()
        ));
    }

    #[test]
    fn analysis_error_returns_none_when_analysis_has_not_finished() {
        let (_, analysis) = create_analysis();

        assert!(analysis.error().is_none());
    }

    #[test]
    fn analysis_error_returns_error_message_when_analysis_has_failed() {
        let (conn, mut analysis) = create_analysis();
        make_analysis_fail(&conn, &mut analysis, "unknown error");

        assert_eq!(analysis.error(), Some("unknown error"));
    }

    #[test]
    fn analysis_get_error_returns_error_and_does_not_update_status_when_analysis_has_failed() {
        let (conn, mut analysis) = create_analysis();
        make_analysis_fail(&conn, &mut analysis, "unknown error");

        let error = analysis.get_error()
            .expect("get_error() should have succeeded");

        assert_eq!(error, Some("unknown error"));
        assert!(conn.borrow().no_requests_sent());
    }

    #[test]
    fn analysis_get_error_checks_status_when_analysis_has_not_yet_failed() {
        let (conn, mut analysis) = create_analysis();
        conn.borrow_mut().add_response(
            "GET",
            "https://retdec.com/service/api/fileinfo/analyses/ID/status",
            Ok(
                APIResponseBuilder::new()
                    .with_status_code(200)
                    .with_body(br#"{
                        "finished": true,
                        "succeeded": false,
                        "failed": true,
                        "error": "unknown error"
                    }"#)
                    .build()
            )
        );

        let error = analysis.get_error()
            .expect("get_error() should have succeeded");

        assert_eq!(error, Some("unknown error"));
        assert!(conn.borrow().request_sent(
            "GET",
            "https://retdec.com/service/api/fileinfo/analyses/ID/status",
            APIArgumentsBuilder::new()
                .build()
        ));
    }

    #[test]
    fn analysis_wait_until_finished_does_nothing_when_analysis_has_finished() {
        let (conn, mut analysis) = create_analysis();
        make_analysis_succeed(&conn, &mut analysis);

        analysis.wait_until_finished()
            .expect("wait_until_finished() should have succeeded");

        assert!(conn.borrow().no_requests_sent());
    }

    #[test]
    fn analysis_wait_until_finished_updates_status_until_analysis_finishes() {
        let (conn, mut analysis) = create_analysis();
        conn.borrow_mut().add_response(
            "GET",
            "https://retdec.com/service/api/fileinfo/analyses/ID/status",
            Ok(
                APIResponseBuilder::new()
                    .with_status_code(200)
                    .with_body(br#"{
                        "finished": true,
                        "succeeded": true,
                        "failed": false
                    }"#)
                    .build()
            )
        );

        analysis.wait_until_finished()
            .expect("wait_until_finished() should have succeeded");

        assert!(conn.borrow().request_sent(
            "GET",
            "https://retdec.com/service/api/fileinfo/analyses/ID/status",
            APIArgumentsBuilder::new()
                .build()
        ));
    }

    #[test]
    fn analysis_get_output_checks_if_analysis_succeeded_and_returns_its_output() {
        let (conn, mut analysis) = create_analysis();
        make_analysis_succeed(&conn, &mut analysis);
        conn.borrow_mut().add_response(
            "GET",
            "https://retdec.com/service/api/fileinfo/analyses/ID/output",
            Ok(
                APIResponseBuilder::new()
                    .with_status_code(200)
                    .with_body(b"Output from analysis")
                    .build()
            )
        );

        let output = analysis.get_output()
            .expect("get_output() should have succeeded");

        assert_eq!(output, "Output from analysis");
        assert!(conn.borrow().request_sent(
            "GET",
            "https://retdec.com/service/api/fileinfo/analyses/ID/output",
            APIArgumentsBuilder::new()
                .build()
        ));
    }

    #[test]
    fn analysis_get_output_as_file_checks_if_analysis_succeeded_and_returns_its_output() {
        let (conn, mut analysis) = create_analysis();
        make_analysis_succeed(&conn, &mut analysis);
        conn.borrow_mut().add_response(
            "GET",
            "https://retdec.com/service/api/fileinfo/analyses/ID/output",
            Ok(
                APIResponseBuilder::new()
                    .with_status_code(200)
                    .with_file(
                        File::from_content_with_name(
                            b"Output from analysis",
                            "output.txt"
                        )
                    )
                    .build()
            )
        );

        let output_file = analysis.get_output_as_file()
            .expect("get_output_as_file() should have succeeded");

        assert_eq!(output_file.content(), b"Output from analysis");
        assert!(conn.borrow().request_sent(
            "GET",
            "https://retdec.com/service/api/fileinfo/analyses/ID/output",
            APIArgumentsBuilder::new()
                .build()
        ));
    }
}
