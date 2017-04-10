//! Connectivity to `retdec.com`'s API.

use std::collections::HashMap;
use std::io::Read;
use std::path::PathBuf;
use std::str;

use hyper::Url as HyperUrl;
use hyper::client::request::Request as HyperRequest;
use hyper::client::response::Response as HyperResponse;
use hyper::header;
use hyper::method::Method as HyperMethod;
use hyper::net::Fresh;
use json::JsonValue;
use json;
use multipart::client::lazy::Multipart;
use multipart::client::SizedRequest;

use error::Result;
use error::ResultExt;
use settings::Settings;

/// Response from `retdec.com`'s API.
#[derive(Debug, Default)]
pub struct APIResponse {
    status_code: u16,
    status_message: String,
    body: Vec<u8>,
}

impl APIResponse {
    pub fn status_code(&self) -> u16 {
        self.status_code
    }

    pub fn status_message(&self) -> &String {
        &self.status_message
    }

    pub fn succeeded(&self) -> bool {
        self.status_code >= 200 && self.status_code <= 299
    }

    pub fn body(&self) -> &Vec<u8> {
        &self.body
    }

    pub fn body_as_str(&self) -> Result<&str> {
        str::from_utf8(&self.body)
            .chain_err(|| "failed to decode API response body as UTF-8")
    }

    pub fn body_as_string(&self) -> Result<String> {
        Ok(self.body_as_str()?.to_string())
    }

    pub fn body_as_json(&self) -> Result<JsonValue> {
        json::parse(self.body_as_str()?)
            .chain_err(|| "failed to parse API response body as JSON")
    }
}

/// API connection.
pub trait APIConnection {
    fn api_url(&self) -> String;

    fn send_get_request(&mut self,
                        url: &str,
                        args: &HashMap<String, String>) -> Result<APIResponse>;

    fn send_post_request(&mut self,
                         url: &str,
                         args: &HashMap<String, String>,
                         files: &HashMap<String, PathBuf>) -> Result<APIResponse>;
}

/// Wrapper of API connections that automatically verifies that requests
/// succeed.
///
/// It wraps an existing API connection. Then, when a response from a GET/POST
/// request is received, it verifies that the request succeeded. If the request
/// failed, it automatically returns an error.
struct ResponseVerifyingAPIConnection<'a> {
    conn: &'a mut APIConnection,
}

impl<'a> ResponseVerifyingAPIConnection<'a> {
    fn ensure_request_succeeded(&self, response: &APIResponse) -> Result<()> {
        if response.succeeded() {
            return Ok(());
        }

        bail!("request failed HTTP {}: {}",
              response.status_code(),
              response.status_message())
    }
}

impl<'a> APIConnection for ResponseVerifyingAPIConnection<'a> {
    fn api_url(&self) -> String {
        self.conn.api_url()
    }

    fn send_get_request(&mut self,
                        url: &str,
                        args: &HashMap<String, String>) -> Result<APIResponse> {
        let response = self.conn.send_get_request(url, args)?;
        self.ensure_request_succeeded(&response)?;
        Ok(response)
    }

    fn send_post_request(&mut self,
                         url: &str,
                         args: &HashMap<String, String>,
                         files: &HashMap<String, PathBuf>) -> Result<APIResponse> {
        let response = self.conn.send_post_request(url, args, files)?;
        self.ensure_request_succeeded(&response)?;
        Ok(response)
    }
}

/// Factory for creating new API connections.
pub trait APIConnectionFactory {
    fn new_connection(&self) -> Box<APIConnection>;
}

/// Connection to `retdec.com`'s API via [hyper](https://hyper.rs/).
pub struct HyperAPIConnection {
    settings: Settings,
}

impl HyperAPIConnection {
    fn prepare_request(&self,
                       method: HyperMethod,
                       url: &str,
                       args: &HashMap<String, String>) -> Result<HyperRequest<Fresh>> {
        let mut parsed_url = HyperUrl::parse(&url)
            .chain_err(|| "invalid API URL")?;
        for (key, value) in args {
            parsed_url.query_pairs_mut().append_pair(key, value);
        }
        let mut request = HyperRequest::<Fresh>::new(method, parsed_url)
            .chain_err(|| format!("failed to create a new request to {}", url))?;
        self.add_auth_to_request(&mut request)?;
        Ok(request)
    }

    fn add_auth_to_request(&self, request: &mut HyperRequest<Fresh>) -> Result<()> {
        // We have to authenticate ourselves by using the API key, which should
        // be passed as 'username' in HTTP Basic Auth. The 'password' part
        // should be left empty.
        // https://retdec.com/api/docs/essential_information.html#authentication
        let auth = header::Authorization(
            header::Basic {
                username: self.settings.api_key()
                    .ok_or("missing API key")?,
                password: None
            }
        );
        request.headers_mut().set(auth);
        Ok(())
    }

    fn parse_response(&self, mut response: HyperResponse) -> Result<APIResponse> {
        let mut body: Vec<u8> = Vec::new();
        response.read_to_end(&mut body)
            .chain_err(|| format!("failed to read the body of a response from {}", response.url))?;
        let raw_status = response.status_raw();
        Ok(APIResponse {
            status_code: raw_status.0,
            status_message: raw_status.1.clone().into_owned(),
            body: body,
        })
    }
}

impl APIConnection for HyperAPIConnection {
    fn api_url(&self) -> String {
        self.settings.api_url()
    }

    fn send_get_request(&mut self,
                        url: &str,
                        args: &HashMap<String, String>) -> Result<APIResponse> {
        let request = self.prepare_request(HyperMethod::Get, &url, &args)
            .chain_err(|| format!("failed to prepare a GET request to {}", url))?;
        let response = request.start()
            .chain_err(|| format!("failed to start a GET request to {}", url))?
            .send()
            .chain_err(|| format!("failed to send a GET request to {}", url))?;
        self.parse_response(response)
    }

    fn send_post_request(&mut self,
                         url: &str,
                         args: &HashMap<String, String>,
                         files: &HashMap<String, PathBuf>) -> Result<APIResponse> {
        let request = self.prepare_request(HyperMethod::Post, &url, &args)
            .chain_err(|| format!("failed to prepare a POST request to {}", url))?;
        let mut mp = Multipart::new();
        for (name, file) in files {
            mp.add_file(name.clone(), file.to_str().ok_or(format!("cannot convert {:?} to str", file))?);
        }
        // The retdec.com API does not support chunked requests, so ensure that
        // we send a request with the Content-Length header.
        // https://retdec.com/api/docs/essential_information.html#transfer-encoding
        let request = SizedRequest::from_request(request);
        let response = mp.send(request)
            .chain_err(|| format!("failed to send a POST request to {}", url))?;
        self.parse_response(response)
    }
}

/// Factory for creating new API connections via [hyper](https://hyper.rs/).
pub struct HyperAPIConnectionFactory {
    settings: Settings,
}

impl HyperAPIConnectionFactory {
    pub fn new(settings: Settings) -> Self {
        HyperAPIConnectionFactory {
            settings: settings,
        }
    }
}

impl APIConnectionFactory for HyperAPIConnectionFactory {
    fn new_connection(&self) -> Box<APIConnection> {
        Box::new(
            HyperAPIConnection {
                settings: self.settings.clone()
            }
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct APIResponseBuilder {
        response: APIResponse,
    }

    impl APIResponseBuilder {
        fn new() -> Self {
            APIResponseBuilder {
                response: APIResponse::default(),
            }
        }

        fn with_status_code(mut self, new_status_code: u16) -> Self {
            self.response.status_code = new_status_code;
            self
        }

        fn with_body(mut self, new_body: &[u8]) -> Self {
            self.response.body = new_body.to_vec();
            self
        }

        fn build(self) -> APIResponse {
            self.response
        }
    }

    #[test]
    fn api_response_getters_return_correct_values() {
        let r = APIResponse {
            status_code: 200,
            status_message: "OK".into(),
            body: "Hello!".into(),
        };

        assert_eq!(r.status_code(), 200);
        assert_eq!(*r.status_message(), "OK".to_string());
        assert_eq!(r.body(), b"Hello!");
    }

    #[test]
    fn api_response_succeeded_returns_true_when_response_succeeded() {
        let r = APIResponseBuilder::new()
            .with_status_code(200)
            .build();

        assert!(r.succeeded());
    }

    #[test]
    fn api_response_succeeded_returns_false_when_response_failed() {
        let r = APIResponseBuilder::new()
            .with_status_code(404)
            .build();

        assert!(!r.succeeded());
    }

    #[test]
    fn api_response_body_as_str_returns_correct_representation() {
        let r = APIResponseBuilder::new()
            .with_body(b"Hello!")
            .build();

        assert_eq!(r.body_as_str().unwrap(), "Hello!");
    }

    #[test]
    fn api_response_body_as_string_returns_correct_representation() {
        let r = APIResponseBuilder::new()
            .with_body(b"Hello!")
            .build();

        assert_eq!(r.body_as_string().unwrap(), "Hello!".to_string());
    }

    #[test]
    fn api_response_body_as_json_returns_correct_representation() {
        let r = APIResponseBuilder::new()
            .with_body(br#"{ "count": 1 }"#)
            .build();

        assert_eq!(r.body_as_json().unwrap()["count"], 1);
    }
}
