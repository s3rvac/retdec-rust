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
    pub error: Option<String>,
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
            error: None,
        }
    }

    /// Updates the status of the resource.
    pub fn update_status(&mut self) -> Result<JsonValue> {
        let err = format!("{} returned invalid JSON response", self.status_url);
        let response = self.conn.send_get_request_without_args(&self.status_url)?;
        let status = response.body_as_json()?;
        self.finished = status["finished"].as_bool().ok_or_else(|| err.clone())?;
        self.succeeded = status["succeeded"].as_bool().ok_or_else(|| err.clone())?;
        self.failed = status["failed"].as_bool().ok_or_else(|| err.clone())?;
        if let Some(error) = status["error"].as_str() {
            self.error = Some(error.to_string());
        }
        Ok(status)
    }

    /// Has the resource finished?
    pub fn has_finished(&mut self) -> Result<bool> {
        self.update_status_if_not_finished()?;
        Ok(self.finished)
    }

    /// Has the resource succeeded?
    pub fn has_succeeded(&mut self) -> Result<bool> {
        self.update_status_if_not_finished()?;
        Ok(self.succeeded)
    }

    /// Has the resource failed?
    pub fn has_failed(&mut self) -> Result<bool> {
        self.update_status_if_not_finished()?;
        Ok(self.failed)
    }

    /// Returns the error message (if any).
    pub fn error(&self) -> Option<&str> {
        self.error.as_ref().map(String::as_str)
    }

    /// Returns the error message (if any).
    pub fn get_error(&mut self) -> Result<Option<&str>> {
        self.update_status_if_not_finished()?;
        Ok(self.error())
    }

    /// Waits (sleeps) for the given time duration.
    pub fn wait_for(&self, duration: Duration) {
        thread::sleep(duration);
    }

    /// Returns an error when the resource failed.
    pub fn ensure_has_succeeded(&mut self, resource_name: &str) -> Result<()> {
        if self.has_succeeded()? {
            Ok(())
        } else {
            bail!("{} has not succeeded", resource_name)
        }
    }

    fn update_status_if_not_finished(&mut self) -> Result<()> {
        if !self.finished {
            self.update_status()?;
        }
        Ok(())
    }
}
