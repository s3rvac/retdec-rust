//! Settings for the provided services.

use std::env;

const DEFAULT_API_URL: &'static str = "https://retdec.com/service/api";

/// Settings for the provided services.
///
/// To use any of the services (decompiler, fileinfo), you have to provide your
/// own API key either by calling `with_api_key()` or by setting the
/// `RETDEC_API_KEY` environment variable.
///
/// # Examples
///
/// ```
/// use retdec::settings::Settings;
///
/// let s = Settings::new()
///     .with_api_key("MY-API-KEY");
///
/// assert_eq!(s.api_key(), Some(&"MY-API-KEY".to_string()));
/// // The API URL is provided by default:
/// assert_eq!(s.api_url(), &"https://retdec.com/service/api");
/// ```
#[derive(Debug, Clone)]
pub struct Settings {
    api_key: Option<String>,
    api_url: String,
}

impl Settings {
    /// Creates new settings.
    ///
    /// The default values depend on whether the following two environment
    /// variables are set:
    ///
    /// * `RETDEC_API_KEY`: If set, its value will be used as the default API
    ///   key. Otherwise, no API key will be set and you have to call
    ///   `with_api_key()` to set it.
    /// * `RETDEC_API_URL`: If set, its value will be used as the default.
    ///   Otherwise, the default API URL is used. For public use, the default
    ///   URL is what you want. Setting a custom API URL is only useful for
    ///   internal development.
    pub fn new() -> Self {
        Settings {
            api_key: Self::default_api_key(),
            api_url: Self::default_api_url(),
        }
    }

    /// Sets an API key.
    ///
    /// Without setting an API key, you will be unable to use any of the
    /// provided services (decompiler, fileinfo).
    pub fn with_api_key<K: Into<String>>(mut self, new_api_key: K) -> Self {
        self.api_key = Some(new_api_key.into());
        self
    }

    /// Sets a custom URL to the API.
    ///
    /// For public use, the default URL is what you want. This function is only
    /// useful for internal development.
    pub fn with_api_url<U: Into<String>>(mut self, new_api_url: U) -> Self {
        self.api_url = new_api_url.into();
        self
    }

    /// Returns the API key.
    ///
    /// If no API key was set, it returns `None`.
    pub fn api_key(&self) -> Option<&String> {
        self.api_key.as_ref()
    }

    /// Returns the API URL.
    pub fn api_url(&self) -> &String {
        &self.api_url
    }

    fn default_api_key() -> Option<String> {
        match env::var("RETDEC_API_KEY") {
            Ok(api_key) => Some(api_key),
            Err(_) => None,
        }
    }

    fn default_api_url() -> String {
        match env::var("RETDEC_API_URL") {
            Ok(api_url) => api_url,
            Err(_) => DEFAULT_API_URL.to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn settings_new_returns_settings_with_default_values() {
        let s = Settings::new();

        // The default values depend on the presence of environment variables.
        match env::var("RETDEC_API_KEY") {
            Ok(api_key) => assert_eq!(s.api_key(), Some(&api_key)),
            Err(_) => assert!(s.api_key().is_none()),
        }
        match env::var("RETDEC_API_URL") {
            Ok(api_url) => assert_eq!(s.api_url(), &api_url),
            Err(_) => assert_eq!(s.api_url(), &DEFAULT_API_URL),
        }
    }

    #[test]
    fn settings_api_key_returns_correct_value_after_being_set() {
        let s = Settings::new()
            .with_api_key("KEY");

        assert_eq!(s.api_key(), Some(&"KEY".to_string()));
    }

    #[test]
    fn settings_api_url_returns_correct_value_after_being_set() {
        let s = Settings::new()
            .with_api_url("URL");

        assert_eq!(*s.api_url(), "URL");
    }

    #[test]
    fn settings_can_set_all_attributes_at_once() {
        let s = Settings::new()
            .with_api_key("KEY")
            .with_api_url("URL");

        assert_eq!(s.api_key(), Some(&"KEY".to_string()));
        assert_eq!(*s.api_url(), "URL");
    }
}
