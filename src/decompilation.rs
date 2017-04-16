//! Decompilations from the decompiler service.

use std::path::PathBuf;
use std::thread;

use connection::APIConnection;
use error::Result;

/// Arguments for a decompilation.
///
/// # Examples
///
/// ```
/// use std::path::Path;
///
/// use retdec::decompilation::DecompilationArguments;
///
/// let args = DecompilationArguments::new()
///     .with_input_file(Path::new("file.exe").to_path_buf());
/// ```
#[derive(Debug, Default)]
pub struct DecompilationArguments {
    output_format: Option<String>,
    verbose: Option<bool>,
    input_file: Option<PathBuf>,
}

impl DecompilationArguments {
    /// Returns new arguments initialized to default values.
    pub fn new() -> Self {
        DecompilationArguments::default()
    }

    /// Sets the path to the file to be analyzed.
    ///
    /// This parameter is required. Without it, there is nothing to analyze.
    pub fn with_input_file(mut self, input_file: PathBuf) -> Self {
        self.input_file = Some(input_file);
        self
    }

    /// Returns the path to the file to be analyzed.
    pub fn input_file(&self) -> Option<&PathBuf> {
        self.input_file.as_ref()
    }
}

/// Decompilation from the decompiler service.
pub struct Decompilation {
    id: String,
    conn: Box<APIConnection>,
}

impl Decompilation {
    /// Creates access to an decompilation with the given ID.
    ///
    /// Only for internal use.
    pub fn new<I: Into<String>>(id: I, conn: Box<APIConnection>) -> Self {
        Decompilation {
            id: id.into(),
            conn: conn,
        }
    }

    /// Returns the ID of the decompilation.
    pub fn id(&self) -> &String {
        &self.id
    }

    /// Waits until the decompilation is finished.
    ///
    /// When this method returns and the result is `Ok()`, the decompilation
    /// has finished.
    pub fn wait_until_finished(&mut self) -> Result<()> {
        loop {
            thread::sleep(::std::time::Duration::from_millis(500));

            let status_url = format!("{}/decompiler/decompilations/{}/status", self.conn.api_url(), self.id);
            let response = self.conn.send_get_request_without_args(&status_url)?;
            let content = response.body_as_json()?;
            let finished = content["finished"].as_bool()
                .ok_or(format!("{} returned invalid JSON response", status_url))?;
            if finished {
                break;
            }
        }
        Ok(())
    }

    /// Returns the content of the output HLL file (C, Python').
    ///
    /// This function should be called only after the decompilation has
    /// finished.
    ///
    /// May access the API.
    pub fn get_output_hll(&mut self) -> Result<String> {
        let output_url = format!(
            "{}/decompiler/decompilations/{}/outputs/hll",
            self.conn.api_url(),
            self.id
        );
        let response = self.conn.send_get_request_without_args(&output_url)?;
        response.body_as_string()
    }
}
