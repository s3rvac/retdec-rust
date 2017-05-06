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
        self.ensure_decompilation_succeeded()?;
        let output_url = format!("{}/outputs/{}", self.resource.base_url, output_type);
        self.resource.conn.send_get_request_without_args(&output_url)
    }

    fn ensure_decompilation_succeeded(&self) -> Result<()> {
        self.resource.ensure_succeeded("decompilation")
    }
}
