//! Decompilations from the decompiler service.

use std::time::Duration;

use connection::APIConnection;
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
#[derive(Debug, Default)]
pub struct DecompilationArguments {
    output_format: Option<String>,
    verbose: Option<bool>,
    input_file: Option<File>,
}

impl DecompilationArguments {
    /// Returns new arguments initialized to default values.
    pub fn new() -> Self {
        DecompilationArguments::default()
    }

    /// Sets the file to be analyzed.
    ///
    /// This parameter is required. Without it, there is nothing to analyze.
    pub fn with_input_file(mut self, input_file: File) -> Self {
        self.input_file = Some(input_file);
        self
    }

    /// Returns the the file to be analyzed.
    pub fn input_file(&self) -> Option<&File> {
        self.input_file.as_ref()
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
    /// might not have finished.
    pub fn finished(&self) -> bool {
        self.resource.finished
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
                .chain_err(|| "failed to update analysis status")?;
            if self.finished() {
                break;
            }
        }
        Ok(())
    }

    /// Returns the content of the output HLL file (e.g. C code).
    ///
    /// This function should be called only after the decompilation has
    /// finished.
    ///
    /// Accesses the API.
    pub fn get_output_hll_code(&mut self) -> Result<String> {
        self.ensure_decompilation_succeeded()?;
        let output_url = format!("{}/outputs/hll", self.resource.base_url);
        let response = self.resource.conn.send_get_request_without_args(&output_url)?;
        response.body_as_string()
    }

    fn ensure_decompilation_succeeded(&self) -> Result<()> {
        self.resource.ensure_succeeded("decompilation")
    }
}
