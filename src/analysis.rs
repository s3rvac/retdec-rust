//! Analyses from the fileinfo service.

use std::time::Duration;

use connection::APIConnection;
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
#[derive(Debug, Default)]
pub struct AnalysisArguments {
    output_format: Option<String>,
    verbose: Option<bool>,
    input_file: Option<File>,
}

impl AnalysisArguments {
    /// Returns new arguments initialized to default values.
    pub fn new() -> Self {
        AnalysisArguments::default()
    }

    /// Sets the format of the output from the analysis.
    ///
    /// Available values are: `plain` (default), `json`.
    pub fn with_output_format(mut self, output_format: &str) -> Self {
        self.output_format = Some(output_format.to_string());
        self
    }

    /// Should the analysis return all available information about the file?
    ///
    /// By default, the analysis returns only an abridged version.
    pub fn with_verbose(mut self, verbose: bool) -> Self {
        self.verbose = Some(verbose);
        self
    }

    /// Sets the file to be analyzed.
    ///
    /// This parameter is required. Without it, there is nothing to analyze.
    pub fn with_input_file(mut self, input_file: File) -> Self {
        self.input_file = Some(input_file);
        self
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
}

/// Analysis from the fileinfo service.
pub struct Analysis {
    resource: Resource,
}

impl Analysis {
    /// Creates access to an analysis with the given ID.
    ///
    /// Only for internal use.
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
        self.ensure_analysis_succeeded()?;
        let output_url = format!("{}/output", self.resource.base_url);
        let response = self.resource.conn.send_get_request_without_args(&output_url)?;
        response.body_as_string()
    }

    fn ensure_analysis_succeeded(&self) -> Result<()> {
        self.resource.ensure_succeeded("analysis")
    }
}
