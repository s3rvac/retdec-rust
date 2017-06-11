//! Access to the testing service
//! ([test](https://retdec.com/api/docs/test.html)).

use std::collections::HashMap;

use connection::APIArguments;
use connection::APIConnectionFactory;
use connection::HyperAPIConnectionFactory;
use error::Result;
use settings::Settings;

/// Testing service.
pub struct Test {
    conn_factory: Box<APIConnectionFactory>,
}

impl Test {
    /// Creates a new instance of the testing service.
    pub fn new(settings: Settings) -> Self {
        Test {
            conn_factory: Box::new(
                HyperAPIConnectionFactory::new(settings)
            ),
        }
    }

    /// Tries to authenticate to the `retdec.com`'s API.
    ///
    /// Returns `Ok(())` when the authentication succeeds. Otherwise, it
    /// returns an error.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use retdec::error::Result;
    /// # fn test() -> Result<()> {
    /// use retdec::settings::Settings;
    /// use retdec::test::Test;
    ///
    /// let settings = Settings::new()
    ///     .with_api_key("MY-API-KEY");
    /// let test = Test::new(settings);
    /// test.auth()?;
    /// # Ok(()) } fn main() { test().unwrap() }
    /// ```
    pub fn auth(&self) -> Result<()> {
        let mut conn = self.conn_factory.new_connection();
        let url = format!("{}/test", conn.api_url());
        let response = conn.send_get_request_without_args(&url)?;
        if response.succeeded() {
            return Ok(());
        } else if response.status_code() == 401 {
            bail!("authentication failed");
        }

        bail!("request to {} failed: {}", url, response.error_reason());
    }

    /// Echoes back the given parameters (key-value pairs).
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use retdec::error::Result;
    /// # fn test() -> Result<()> {
    /// use std::collections::HashMap;
    /// use retdec::settings::Settings;
    /// use retdec::test::Test;
    ///
    /// let settings = Settings::new()
    ///     .with_api_key("MY-API-KEY");
    /// let test = Test::new(settings);
    /// let mut params = HashMap::new();
    /// params.insert("param1".to_string(), "value1".to_string());
    /// params.insert("param2".to_string(), "value2".to_string());
    /// let result = test.echo(&params)?;
    /// assert_eq!(result.get("param1"), Some(&"value1".to_string()));
    /// assert_eq!(result.get("param2"), Some(&"value2".to_string()));
    /// # Ok(()) } fn main() { test().unwrap() }
    /// ```
    pub fn echo(&self, params: &HashMap<String, String>)
        -> Result<HashMap<String, String>>
    {
        let mut conn = self.conn_factory.new_connection();
        let url = format!("{}/test/echo", conn.api_url());
        let mut args = APIArguments::new();
        for (key, value) in params {
            args.add_string_arg(key.as_str(), value.as_str());
        }
        let response = conn.send_get_request(&url, args)?;
        if response.failed() {
            bail!("request to {} failed: {}", url, response.error_reason());
        }

        let mut out_params = HashMap::new();
        let json = response.body_as_json()?;
        for (key, value) in json.entries() {
            let value = value.as_str()
                .ok_or_else(|| format!("{} returned invalid JSON response", url))?;
            out_params.insert(key.to_string(), value.to_string());
        }
        Ok(out_params)
    }

    #[cfg(test)]
    fn with_conn_factory(conn_factory: Box<APIConnectionFactory>) -> Self {
        Test { conn_factory: conn_factory }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::cell::RefCell;
    use std::rc::Rc;

    use connection::tests::APIArgumentsBuilder;
    use connection::tests::APIConnectionFactoryMock;
    use connection::tests::APIConnectionMock;
    use connection::tests::APIResponseBuilder;

    fn create_test() -> (Rc<RefCell<APIConnectionMock>>, Test) {
        // We need to force an API URL to prevent it from being overridden by
        // setting the RETDEC_API_URL environment variable.
        let settings = Settings::new()
            .with_api_key("test")
            .with_api_url("https://retdec.com/service/api");
        let conn = Rc::new(RefCell::new(APIConnectionMock::new(settings.clone())));
        let conn_factory = Box::new(APIConnectionFactoryMock::new(conn.clone()));
        (conn, Test::with_conn_factory(conn_factory))
    }

    #[test]
    fn auth_returns_unit_when_auth_succeeds() {
        let (conn, test) = create_test();
        conn.borrow_mut().add_response(
            "GET",
            "https://retdec.com/service/api/test",
            Ok(
                APIResponseBuilder::new()
                    .with_status_code(200)
                    .with_body(b"{}")
                    .build()
            )
        );

        let result = test.auth();

        assert!(result.is_ok());
    }

    #[test]
    fn auth_returns_error_when_auth_fails() {
        let (conn, test) = create_test();
        conn.borrow_mut().add_response(
            "GET",
            "https://retdec.com/service/api/test",
            Ok(
                APIResponseBuilder::new()
                    .with_status_code(401)
                    .with_body(br#"{
                        "code": 401,
                        "description": "API key authorization failed.",
                        "message": "Unauthorized by API Key"
                    }"#)
                    .build()
            )
        );

        let result = test.auth();

        let err = result.err().expect("expected auth() to fail");
        assert_eq!(err.to_string(), "authentication failed");
    }

    #[test]
    fn auth_returns_error_when_request_fails() {
        let (conn, test) = create_test();
        conn.borrow_mut().add_response(
            "GET",
            "https://retdec.com/service/api/test",
            Ok(
                APIResponseBuilder::new()
                    .with_status_code(404)
                    .build()
            )
        );

        let result = test.auth();

        let err = result.err().expect("expected auth() to fail");
        assert!(err.to_string().contains("HTTP 404"));
    }

    #[test]
    fn echo_returns_back_input_parameters_when_request_succeeds() {
        let (conn, test) = create_test();
        conn.borrow_mut().add_response(
            "GET",
            "https://retdec.com/service/api/test/echo",
            Ok(
                APIResponseBuilder::new()
                    .with_status_code(200)
                    .with_body(br#"{
                        "param1": "value1",
                        "param2": "value2"
                    }"#)
                    .build()
            )
        );
        let mut params = HashMap::new();
        params.insert("param1".to_string(), "value1".to_string());
        params.insert("param2".to_string(), "value2".to_string());

        let result = test.echo(&params)
            .expect("expected echo() to succeed");

        assert_eq!(result.get("param1"), Some(&"value1".to_string()));
        assert_eq!(result.get("param2"), Some(&"value2".to_string()));
        assert!(conn.borrow().request_sent(
            "GET",
            "https://retdec.com/service/api/test/echo",
            APIArgumentsBuilder::new()
                .with_string_arg("param1", "value1")
                .with_string_arg("param2", "value2")
                .build()
        ));
    }
}
