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
/// assert_eq!(s.api_key(), Some("MY-API-KEY"));
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

    /// Sets an API key when used as a builder.
    ///
    /// Without setting an API key, you will be unable to use any of the
    /// provided services (decompiler, fileinfo).
    ///
    /// # Examples
    ///
    /// ```
    /// use retdec::settings::Settings;
    ///
    /// let s = Settings::new()
    ///     .with_api_key("MY-API-KEY");
    ///
    /// assert_eq!(s.api_key(), Some("MY-API-KEY"));
    /// ```
    pub fn with_api_key<K: Into<String>>(mut self, new_api_key: K) -> Self {
        self.set_api_key(new_api_key);
        self
    }

    /// Sets a custom URL to the API when used as a builder.
    ///
    /// For public use, the default URL is what you want. This function is only
    /// useful for internal development.
    pub fn with_api_url<U: Into<String>>(mut self, new_api_url: U) -> Self {
        self.set_api_url(new_api_url);
        self
    }

    /// Sets an API key.
    ///
    /// Without setting an API key, you will be unable to use any of the
    /// provided services (decompiler, fileinfo).
    ///
    /// # Examples
    ///
    /// ```
    /// use retdec::settings::Settings;
    ///
    /// let mut s = Settings::new();
    /// s.set_api_key("MY-API-KEY");
    ///
    /// assert_eq!(s.api_key(), Some("MY-API-KEY"));
    /// ```
    pub fn set_api_key<K: Into<String>>(&mut self, new_api_key: K) {
        self.api_key = Some(new_api_key.into());
    }

    /// Sets a custom URL to the API.
    ///
    /// For public use, the default URL is what you want. This function is only
    /// useful for internal development.
    pub fn set_api_url<U: Into<String>>(&mut self, new_api_url: U) {
        self.api_url = Self::normalize_api_url(new_api_url.into());
    }

    /// Returns the API key.
    ///
    /// If no API key was set, it returns `None`.
    pub fn api_key(&self) -> Option<&str> {
        self.api_key.as_ref().map(String::as_str)
    }

    /// Returns the API URL.
    pub fn api_url(&self) -> &str {
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
            Ok(api_url) => Self::normalize_api_url(api_url),
            Err(_) => DEFAULT_API_URL.to_string(),
        }
    }

    fn normalize_api_url(mut api_url: String) -> String {
        // We need to ensure that the URL does not end with a slash because the
        // retdec.com'a API does not use trailing slashes. This simplifies the
        // use of Settings::api_url() in the library.
        if api_url.ends_with('/') {
            api_url.pop();
        }
        api_url
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
            Ok(api_key) => assert_eq!(s.api_key(), Some(api_key.as_str())),
            Err(_) => assert!(s.api_key().is_none()),
        }
        match env::var("RETDEC_API_URL") {
            Ok(mut api_url) => {
                if api_url.ends_with('/') {
                    api_url.pop();
                }
                assert_eq!(s.api_url(), &api_url)
            }
            Err(_) => assert_eq!(s.api_url(), DEFAULT_API_URL),
        }
    }

    #[test]
    fn settings_api_key_returns_correct_value_after_being_set() {
        let mut s = Settings::new();
        s.set_api_key("KEY");

        assert_eq!(s.api_key(), Some("KEY"));
    }

    #[test]
    fn settings_api_url_returns_correct_value_after_being_set() {
        let mut s = Settings::new();
        s.set_api_url("URL");

        assert_eq!(s.api_url(), "URL");
    }

    #[test]
    fn settings_trailing_slash_is_removed_from_api_url() {
        let s = Settings::new()
            .with_api_url(format!("{}/", DEFAULT_API_URL));

        assert_eq!(s.api_url(), DEFAULT_API_URL);
    }

    #[test]
    fn settings_can_set_all_attributes_at_once_via_with_methods() {
        let s = Settings::new()
            .with_api_key("KEY")
            .with_api_url("URL");

        assert_eq!(s.api_key(), Some("KEY"));
        assert_eq!(s.api_url(), "URL");
    }
}
