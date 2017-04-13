//! Analyses from the fileinfo service.

use std::collections::HashMap;
use std::path::PathBuf;
use std::thread;

use connection::APIConnection;
use error::Result;

/// Arguments for an analysis.
#[derive(Default)]
pub struct AnalysisArguments {
    verbose: Option<bool>,
    input_file: Option<PathBuf>,
}

impl AnalysisArguments {
    pub fn new() -> Self {
        AnalysisArguments::default()
    }

    pub fn with_input_file(mut self, input_file: PathBuf) -> Self {
        self.input_file = Some(input_file);
        self
    }

    pub fn with_verbose(mut self, verbose: bool) -> Self {
        self.verbose = Some(verbose);
        self
    }

    pub fn verbose(&self) -> Option<bool> {
        self.verbose
    }

    pub fn input_file(&self) -> Option<PathBuf> {
        self.input_file.clone()
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
            let args = HashMap::new();
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
        let args = HashMap::new();
        let response = self.conn.send_get_request(&output_url, &args)?;
        response.body_as_string()
    }
}
