//! Decompilations from the decompiler service.

use std::time::Duration;

use connection::APIConnection;
use connection::APIResponse;
use error::Result;
use error::ResultExt;
use file::File;
use resource::Resource;

/// Arguments for a decompilation.
///
/// # Examples
///
/// ```no_run
/// use retdec::file::File;
/// use retdec::decompilation::DecompilationArguments;
///
/// let args = DecompilationArguments::new()
///     .with_input_file(File::from_path("file.exe").unwrap());
/// ```
#[derive(Clone, Debug, Default)]
pub struct DecompilationArguments {
    input_file: Option<File>,
}

impl DecompilationArguments {
    /// Returns new arguments initialized to default values.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the file to be decompiled.
    ///
    /// This parameter is required. Without it, there is nothing to decompile.
    pub fn with_input_file(mut self, input_file: File) -> Self {
        self.set_input_file(input_file);
        self
    }

    /// Sets the file to be analyzed.
    ///
    /// This parameter is required. Without it, there is nothing to analyze.
    pub fn set_input_file(&mut self, input_file: File) {
        self.input_file = Some(input_file);
    }

    /// Returns the the file to be decompiled.
    pub fn input_file(&self) -> Option<&File> {
        self.input_file.as_ref()
    }

    /// Takes ownership of the input file and sets it to `None`.
    pub fn take_input_file(&mut self) -> Option<File> {
        self.input_file.take()
    }
}

/// Decompilation from the decompiler service.
pub struct Decompilation {
    resource: Resource,
}

impl Decompilation {
    /// Creates access to an decompilation with the given ID.
    ///
    /// Only for internal use.
    #[doc(hidden)]
    pub fn new<I: Into<String>>(id: I, conn: Box<APIConnection>) -> Self {
        Decompilation {
            resource: Resource::new("decompiler", "decompilations", id, conn)
        }
    }

    /// Returns the ID of the decompilation.
    ///
    /// Does not access the API.
    pub fn id(&self) -> &String {
        &self.resource.id
    }

    /// Has the decompilation finished?
    ///
    /// Does not access the API, so the returned value may be outdated. In a
    /// greater detail, when it returns `true`, the decompilation has surely
    /// finished. However, when it returns `false`, the decompilation might or
    /// might not have finished. If you want to have an up-to-date information,
    /// use `has_finished()` instead.
    pub fn finished(&self) -> bool {
        self.resource.finished
    }

    /// Has the decompilation finished?
    ///
    /// Accesses the API.
    pub fn has_finished(&mut self) -> Result<bool> {
        self.resource.has_finished()
    }

    /// Has the decompilation succeeded?
    ///
    /// Does not access the API, so the returned value may be outdated. If you
    /// want to have an up-to-date information, use `has_succeeded()` instead.
    ///
    /// The returned value makes sense only when the decompilation has
    /// finished.
    pub fn succeeded(&self) -> bool {
        self.resource.finished
    }

    /// Has the decompilation succeeded?
    ///
    /// Accesses the API.
    ///
    /// The returned value makes sense only when the decompilation has
    /// finished.
    pub fn has_succeeded(&mut self) -> Result<bool> {
        self.resource.has_succeeded()
    }

    /// Has the decompilation failed?
    ///
    /// Does not access the API, so the returned value may be outdated. If you
    /// want to have an up-to-date information, use `has_failed()` instead.
    ///
    /// The returned value makes sense only when the decompilation has
    /// finished.
    pub fn failed(&self) -> bool {
        self.resource.finished
    }

    /// Has the decompilation failed?
    ///
    /// Accesses the API.
    ///
    /// The returned value makes sense only when the decompilation has
    /// finished.
    pub fn has_failed(&mut self) -> Result<bool> {
        self.resource.has_failed()
    }

    /// Returns the error message (if any).
    ///
    /// Does not access the API, so the returned value may be outdated. If you
    /// want to have an up-to-date information, use `get_error()` instead.
    ///
    /// Calling this method makes sense only when the decompilation has failed.
    /// Otherwise, it will always return `None`.
    pub fn error(&self) -> Option<&str> {
        self.resource.error()
    }

    /// Returns the error message (if any).
    ///
    /// Accesses the API.
    ///
    /// Calling this method makes sense only when the decompilation has failed.
    /// Otherwise, it will always return `None`.
    pub fn get_error(&mut self) -> Result<Option<&str>> {
        self.resource.get_error()
    }

    /// Waits until the decompilation has finished.
    ///
    /// When this method returns `Ok()`, the decompilation has finished.
    ///
    /// Accesses the API.
    pub fn wait_until_finished(&mut self) -> Result<()> {
        // Currently, the retdec.com's API does not support push notifications,
        // so we have to poll for the status ourselves.
        while !self.finished() {
            self.resource.wait_for(Duration::from_millis(500));

            self.resource.update_status()
                .chain_err(|| "failed to update decompilation status")?;
            if self.finished() {
                break;
            }
        }
        Ok(())
    }

    /// Returns the output code in the target high-level language (HLL).
    ///
    /// The HLL type (C, Python') depends on the used decompilation arguments.
    ///
    /// This function should be called only after the decompilation has
    /// successfully finished.
    ///
    /// Accesses the API.
    pub fn get_output_hll_code(&mut self) -> Result<String> {
        let response = self.get_output_response("hll")?;
        response.body_as_string()
    }

    /// Returns the output code in the target high-level language (HLL) as a
    /// file.
    ///
    /// The HLL type (C, Python') depends on the used decompilation arguments.
    ///
    /// This function should be called only after the decompilation has
    /// successfully finished.
    ///
    /// Accesses the API.
    pub fn get_output_hll_code_as_file(&mut self) -> Result<File> {
        let response = self.get_output_response("hll")?;
        response.body_as_file()
    }

    fn get_output_response(&mut self, output_type: &str) -> Result<APIResponse> {
        self.ensure_decompilation_has_succeeded()?;
        let output_url = format!("{}/outputs/{}", self.resource.base_url, output_type);
        self.resource.conn.send_get_request_without_args(&output_url)
    }

    fn ensure_decompilation_has_succeeded(&mut self) -> Result<()> {
        self.resource.ensure_has_succeeded("decompilation")
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

    fn create_decompilation() -> (Rc<RefCell<APIConnectionMock>>, Decompilation) {
        // We need to force an API URL to prevent it from being overriden by
        // setting the RETDEC_API_URL environment variable.
        let settings = Settings::new()
            .with_api_key("test")
            .with_api_url("https://retdec.com/service/api");
        let conn = Rc::new(
            RefCell::new(
                APIConnectionMock::new(settings.clone())
            )
        );
        let conn_wrapper = Box::new(
            APIConnectionMockWrapper::new(settings, conn.clone())
        );
        (conn, Decompilation::new("ID", conn_wrapper))
    }

    fn make_decompilation_succeed(conn: &Rc<RefCell<APIConnectionMock>>,
                                  decompilation: &mut Decompilation) {
        conn.borrow_mut().add_response(
            "GET",
            "https://retdec.com/service/api/decompiler/decompilations/ID/status",
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
        decompilation.wait_until_finished()
            .expect("expected the decompilation to finish successfully");
        assert!(conn.borrow().request_sent(
            "GET",
            "https://retdec.com/service/api/decompiler/decompilations/ID/status",
            APIArgumentsBuilder::new()
                .build()
        ));
        conn.borrow_mut().reset();
    }

    fn make_decompilation_fail(conn: &Rc<RefCell<APIConnectionMock>>,
                               decompilation: &mut Decompilation,
                               error: &str) {
        conn.borrow_mut().add_response(
            "GET",
            "https://retdec.com/service/api/decompiler/decompilations/ID/status",
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
        decompilation.wait_until_finished()
            .expect("expected the decompilation to finish successfully");
        assert!(conn.borrow().request_sent(
            "GET",
            "https://retdec.com/service/api/decompiler/decompilations/ID/status",
            APIArgumentsBuilder::new()
                .build()
        ));
        conn.borrow_mut().reset();
    }

    #[test]
    fn decompilation_id_returns_id_of_decompilation() {
        let (_, decompilation) = create_decompilation();

        assert_eq!(decompilation.id(), "ID");
    }

    #[test]
    fn decompilation_finished_returns_false_when_decompilation_has_not_finished() {
        let (_, decompilation) = create_decompilation();

        assert!(!decompilation.finished());
    }

    #[test]
    fn decompilation_finished_returns_true_when_decompilation_has_finished() {
        let (conn, mut decompilation) = create_decompilation();
        make_decompilation_succeed(&conn, &mut decompilation);

        assert!(decompilation.finished());
    }

    #[test]
    fn decompilation_has_finished_returns_true_and_does_not_update_status_when_decompilation_has_finished() {
        let (conn, mut decompilation) = create_decompilation();
        make_decompilation_succeed(&conn, &mut decompilation);

        let finished = decompilation.has_finished()
            .expect("has_finished() should have succeeded");

        assert!(finished);
        assert!(conn.borrow().no_requests_sent());
    }

    #[test]
    fn decompilation_has_finished_checks_status_when_decompilation_has_not_yet_finished() {
        let (conn, mut decompilation) = create_decompilation();
        conn.borrow_mut().add_response(
            "GET",
            "https://retdec.com/service/api/decompiler/decompilations/ID/status",
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

        let finished = decompilation.has_finished()
            .expect("has_finished() should have succeeded");

        assert!(finished);
        assert!(conn.borrow().request_sent(
            "GET",
            "https://retdec.com/service/api/decompiler/decompilations/ID/status",
            APIArgumentsBuilder::new()
                .build()
        ));
    }

    #[test]
    fn decompilation_succeeded_returns_false_when_decompilation_has_not_yet_finished() {
        let (_, decompilation) = create_decompilation();

        assert!(!decompilation.succeeded());
    }

    #[test]
    fn decompilation_succeeded_returns_true_when_decompilation_has_succeeded() {
        let (conn, mut decompilation) = create_decompilation();
        make_decompilation_succeed(&conn, &mut decompilation);

        assert!(decompilation.succeeded());
    }

    #[test]
    fn decompilation_has_succeeded_returns_true_and_does_not_update_status_when_decompilation_has_succeeded() {
        let (conn, mut decompilation) = create_decompilation();
        make_decompilation_succeed(&conn, &mut decompilation);

        let succeeded = decompilation.has_succeeded()
            .expect("has_succeeded() should have succeeded");

        assert!(succeeded);
        assert!(conn.borrow().no_requests_sent());
    }

    #[test]
    fn decompilation_has_succeeded_checks_status_when_decompilation_has_not_yet_succeeded() {
        let (conn, mut decompilation) = create_decompilation();
        conn.borrow_mut().add_response(
            "GET",
            "https://retdec.com/service/api/decompiler/decompilations/ID/status",
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

        let succeeded = decompilation.has_succeeded()
            .expect("has_succeeded() should have succeeded");

        assert!(succeeded);
        assert!(conn.borrow().request_sent(
            "GET",
            "https://retdec.com/service/api/decompiler/decompilations/ID/status",
            APIArgumentsBuilder::new()
                .build()
        ));
    }

    #[test]
    fn decompilation_failed_returns_false_when_decompilation_has_not_yet_finished() {
        let (_, decompilation) = create_decompilation();

        assert!(!decompilation.failed());
    }

    #[test]
    fn decompilation_failed_returns_true_when_decompilation_has_failed() {
        let (conn, mut decompilation) = create_decompilation();
        make_decompilation_fail(&conn, &mut decompilation, "unknown error");

        assert!(decompilation.failed());
    }

    #[test]
    fn decompilation_has_failed_returns_true_and_does_not_update_status_when_decompilation_has_failed() {
        let (conn, mut decompilation) = create_decompilation();
        make_decompilation_fail(&conn, &mut decompilation, "unknown error");

        let failed = decompilation.has_failed()
            .expect("has_failed() should have succeeded");

        assert!(failed);
        assert!(conn.borrow().no_requests_sent());
    }

    #[test]
    fn decompilation_has_failed_checks_status_when_decompilation_has_not_yet_failed() {
        let (conn, mut decompilation) = create_decompilation();
        conn.borrow_mut().add_response(
            "GET",
            "https://retdec.com/service/api/decompiler/decompilations/ID/status",
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

        let failed = decompilation.has_failed()
            .expect("has_failed() should have succeeded");

        assert!(failed);
        assert!(conn.borrow().request_sent(
            "GET",
            "https://retdec.com/service/api/decompiler/decompilations/ID/status",
            APIArgumentsBuilder::new()
                .build()
        ));
    }

    #[test]
    fn decompilation_error_returns_none_when_decompilation_has_not_finished() {
        let (_, decompilation) = create_decompilation();

        assert!(decompilation.error().is_none());
    }

    #[test]
    fn decompilation_error_returns_error_message_when_decompilation_has_failed() {
        let (conn, mut decompilation) = create_decompilation();
        make_decompilation_fail(&conn, &mut decompilation, "unknown error");

        assert_eq!(decompilation.error(), Some("unknown error"));
    }

    #[test]
    fn decompilation_get_error_returns_error_and_does_not_update_status_when_decompilation_has_failed() {
        let (conn, mut decompilation) = create_decompilation();
        make_decompilation_fail(&conn, &mut decompilation, "unknown error");

        let error = decompilation.get_error()
            .expect("get_error() should have succeeded");

        assert_eq!(error, Some("unknown error"));
        assert!(conn.borrow().no_requests_sent());
    }

    #[test]
    fn decompilation_get_error_checks_status_when_decompilation_has_not_yet_failed() {
        let (conn, mut decompilation) = create_decompilation();
        conn.borrow_mut().add_response(
            "GET",
            "https://retdec.com/service/api/decompiler/decompilations/ID/status",
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

        let error = decompilation.get_error()
            .expect("get_error() should have succeeded");

        assert_eq!(error, Some("unknown error"));
        assert!(conn.borrow().request_sent(
            "GET",
            "https://retdec.com/service/api/decompiler/decompilations/ID/status",
            APIArgumentsBuilder::new()
                .build()
        ));
    }

    #[test]
    fn decompilation_wait_until_finished_does_nothing_when_decompilation_has_finished() {
        let (conn, mut decompilation) = create_decompilation();
        make_decompilation_succeed(&conn, &mut decompilation);

        decompilation.wait_until_finished()
            .expect("wait_until_finished() should have succeeded");

        assert!(conn.borrow().no_requests_sent());
    }

    #[test]
    fn decompilation_wait_until_finished_updates_status_until_decompilation_finishes() {
        let (conn, mut decompilation) = create_decompilation();
        conn.borrow_mut().add_response(
            "GET",
            "https://retdec.com/service/api/decompiler/decompilations/ID/status",
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

        decompilation.wait_until_finished()
            .expect("wait_until_finished() should have succeeded");

        assert!(conn.borrow().request_sent(
            "GET",
            "https://retdec.com/service/api/decompiler/decompilations/ID/status",
            APIArgumentsBuilder::new()
                .build()
        ));
    }

    #[test]
    fn decompilation_get_output_hll_code_checks_if_decompilation_succeeded_and_returns_its_output() {
        let (conn, mut decompilation) = create_decompilation();
        make_decompilation_succeed(&conn, &mut decompilation);
        conn.borrow_mut().add_response(
            "GET",
            "https://retdec.com/service/api/decompiler/decompilations/ID/outputs/hll",
            Ok(
                APIResponseBuilder::new()
                    .with_status_code(200)
                    .with_body(b"Output from decompilation")
                    .build()
            )
        );

        let output = decompilation.get_output_hll_code()
            .expect("get_output_hll_code() should have succeeded");

        assert_eq!(output, "Output from decompilation");
        assert!(conn.borrow().request_sent(
            "GET",
            "https://retdec.com/service/api/decompiler/decompilations/ID/outputs/hll",
            APIArgumentsBuilder::new()
                .build()
        ));
    }

    #[test]
    fn decompilation_get_output_hll_code_as_file_checks_if_decompilation_succeeded_and_returns_its_output() {
        let (conn, mut decompilation) = create_decompilation();
        make_decompilation_succeed(&conn, &mut decompilation);
        conn.borrow_mut().add_response(
            "GET",
            "https://retdec.com/service/api/decompiler/decompilations/ID/outputs/hll",
            Ok(
                APIResponseBuilder::new()
                    .with_status_code(200)
                    .with_file(
                        File::from_content_with_name(
                            b"Output from decompilation",
                            "output.txt"
                        )
                    )
                    .build()
            )
        );

        let output_file = decompilation.get_output_hll_code_as_file()
            .expect("get_output_hll_code_as_file() should have succeeded");

        assert_eq!(output_file.content(), b"Output from decompilation");
        assert!(conn.borrow().request_sent(
            "GET",
            "https://retdec.com/service/api/decompiler/decompilations/ID/outputs/hll",
            APIArgumentsBuilder::new()
                .build()
        ));
    }
}
