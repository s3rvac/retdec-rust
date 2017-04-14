//! Analyses from the fileinfo service.

use std::path::PathBuf;
use std::thread;

use connection::APIArguments;
use connection::APIConnection;
use error::Result;

/// Arguments for an analysis.
#[derive(Debug, Default)]
pub struct AnalysisArguments {
    output_format: Option<String>,
    verbose: Option<bool>,
    input_file: Option<PathBuf>,
}

impl AnalysisArguments {
    pub fn new() -> Self {
        AnalysisArguments::default()
    }

    pub fn with_output_format(mut self, output_format: &str) -> Self {
        self.output_format = Some(output_format.to_string());
        self
    }

    pub fn with_verbose(mut self, verbose: bool) -> Self {
        self.verbose = Some(verbose);
        self
    }

    pub fn with_input_file(mut self, input_file: PathBuf) -> Self {
        self.input_file = Some(input_file);
        self
    }

    pub fn output_format(&self) -> &Option<String> {
        &self.output_format
    }

    pub fn verbose(&self) -> &Option<bool> {
        &self.verbose
    }

    pub fn input_file(&self) -> &Option<PathBuf> {
        &self.input_file
    }
}

/// Analysis from the fileinfo service.
pub struct Analysis {
    id: String,
    conn: Box<APIConnection>,
}

impl Analysis {
    pub fn new(id: &str, conn: Box<APIConnection>) -> Self {
        Analysis {
            id: id.to_string(),
            conn: conn,
        }
    }

    pub fn id(&self) -> &String {
        &self.id
    }

    pub fn wait_until_finished(&mut self) -> Result<()> {
        loop {
            thread::sleep(::std::time::Duration::from_millis(500));

            let status_url = format!("{}/fileinfo/analyses/{}/status", self.conn.api_url(), self.id);
            let args = APIArguments::new();
            let response = self.conn.send_get_request(&status_url, &args)?;
            let content = response.body_as_json()?;
            let finished = content["finished"].as_bool().unwrap();
            if finished {
                break;
            }
        }
        Ok(())
    }

    pub fn get_output(&mut self) -> Result<String> {
        let output_url = format!("{}/fileinfo/analyses/{}/output", self.conn.api_url(), self.id);
        let args = APIArguments::new();
        let response = self.conn.send_get_request(&output_url, &args)?;
        response.body_as_string()
    }
}
