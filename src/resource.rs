//! Common functionality shared by all resources (analyses, decompilations).

use std::thread;
use std::time::Duration;

use json::JsonValue;

use connection::APIConnection;
use error::Result;

/// Access to a resource.
pub struct Resource {
    pub id: String,
    pub conn: Box<APIConnection>,
    pub base_url: String,
    pub status_url: String,
    pub finished: bool,
    pub succeeded: bool,
    pub failed: bool,
    pub error: String,
}

impl Resource {
    /// Creates access to a resource with the given ID.
    pub fn new<I>(
        service_name: &str,
        resources_name: &str,
        id: I,
        conn: Box<APIConnection>,
    ) -> Self
        where I: Into<String>,
    {
        let id = id.into();
        let base_url = format!(
            "{}/{}/{}/{}",
            conn.api_url(),
            service_name,
            resources_name,
            id
        );
        let status_url = format!("{}/status", base_url);

        Resource {
            id: id,
            conn: conn,
            base_url: base_url,
            status_url: status_url,
            finished: false,
            succeeded: false,
            failed: false,
            error: String::default(),
        }
    }

    /// Updates the status of the resource.
    pub fn update_status(&mut self) -> Result<JsonValue> {
        let err = format!("{} returned invalid JSON response", self.status_url);
        let response = self.conn.send_get_request_without_args(&self.status_url)?;
        let status = response.body_as_json()?;
        self.finished = status["finished"].as_bool().ok_or(err.clone())?;
        self.succeeded = status["succeeded"].as_bool().ok_or(err.clone())?;
        self.failed = status["failed"].as_bool().ok_or(err.clone())?;
        if let Some(error) = status["error"].as_str() {
            self.error = error.to_string();
        }
        Ok(status)
    }

    /// Waits (sleeps) for the given time duration.
    pub fn wait_for(&self, duration: Duration) {
        thread::sleep(duration);
    }
}
