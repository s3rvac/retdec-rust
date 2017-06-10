//! Access to the testing service
//! ([test](https://retdec.com/api/docs/test.html)).

use connection::APIConnectionFactory;
use connection::HyperAPIConnectionFactory;
use error::Result;
use settings::Settings;

/// Testing service.
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
}
