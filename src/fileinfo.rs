//! Access to the file-analyzing service
//! ([fileinfo](https://retdec.com/api/docs/fileinfo.html)).

use std::collections::HashMap;

use analysis::Analysis;
use analysis::AnalysisArguments;
use connection::APIConnectionFactory;
use connection::HyperAPIConnectionFactory;
use error::Result;
use error::ResultExt;
use settings::Settings;

/// File-analyzing service.
pub struct Fileinfo {
    conn_factory: Box<APIConnectionFactory>,
}

impl Fileinfo {
    pub fn new(settings: Settings) -> Self {
        Fileinfo {
            conn_factory: Box::new(HyperAPIConnectionFactory::new(settings)),
        }
    }

    pub fn start_analysis(&self, args: AnalysisArguments) -> Result<Analysis> {
        let mut conn = self.conn_factory.new_connection();
        let url = format!("{}/fileinfo/analyses", conn.api_url());
        let mut request_args = HashMap::new();
        if let Some(output_format) = args.output_format() {
            request_args.insert("output_format".to_string(), output_format);
        };
        if let Some(verbose) = args.verbose() {
            request_args.insert("verbose".to_string(), verbose.to_string());
        };
        let mut files = HashMap::new();
        match args.input_file() {
            Some(input_file) => {
                files.insert("input".to_string(), input_file);
            },
            None => {
                bail!("no input file");
            },
        }
        let response = conn.send_post_request(&url, &request_args, &files)
            .chain_err(|| "failed to start an analysis")?;
        let content = response.body_as_json()?;
        let id = content["id"].as_str().unwrap();
        Ok(Analysis::new(id, conn))
    }
}
