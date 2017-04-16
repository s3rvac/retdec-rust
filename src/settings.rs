//! Settings for the provided services.

const DEFAULT_API_URL: &'static str = "https://retdec.com/service/api";

/// Settings for the provided services.
///
/// To use any of the services (decompiler, fileinfo), you have to provide your
/// own API key by calling `with_api_key()`.
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
    pub fn new() -> Self {
        Settings {
            api_key: None,
            api_url: DEFAULT_API_URL.to_string(),
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn settings_new_returns_settings_with_default_values() {
        let s = Settings::new();

        assert!(s.api_key().is_none());
        assert_eq!(s.api_url(), &DEFAULT_API_URL);
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
    fn settings_can_set_all_attributes() {
        let s = Settings::new()
            .with_api_key("KEY")
            .with_api_url("URL");

        assert_eq!(s.api_key(), Some(&"KEY".to_string()));
        assert_eq!(*s.api_url(), "URL");
    }
}
