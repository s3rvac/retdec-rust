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
/// use retdec::analysis::AnalysisArguments;
/// use retdec::file::File;
///
/// let args = AnalysisArguments::new()
///     .with_output_format("json")
///     .with_verbose(true)
///     .with_input_file(File::from_path("file.exe").unwrap());
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
    /// not have finished.
    pub fn finished(&self) -> bool {
        self.resource.finished
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
