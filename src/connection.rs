//! Connectivity to `retdec.com`'s API.

use std::collections::HashMap;
use std::collections::hash_map::Iter as ArgIter;
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

/// Arguments passed to the `retdec.com`s API via GET/HTTP requests.
#[derive(Debug)]
pub struct APIArguments {
    args: HashMap<String, String>,
    files: HashMap<String, PathBuf>,
}

impl APIArguments {
    pub fn new() -> Self {
        APIArguments {
            args: HashMap::new(),
            files: HashMap::new(),
        }
    }

    pub fn add_string_arg<N, V>(&mut self, name: N, value: V)
        where N: Into<String>, V: Into<String>
    {
        self.args.insert(name.into(), value.into());
    }

    pub fn add_opt_string_arg<N>(&mut self, name: N, value: Option<&String>)
        where N: Into<String>
    {
        if let Some(value) = value {
            self.args.insert(name.into(), value.to_string());
        }
    }

    pub fn add_bool_arg<N>(&mut self, name: N, value: bool)
        where N: Into<String>
    {
        let value = if value { 1 } else { 0 };
        self.args.insert(name.into(), value.to_string());
    }

    pub fn add_opt_bool_arg<N>(&mut self, name: N, value: Option<bool>)
        where N: Into<String>
    {
        if let Some(value) = value {
            let value = if value { 1 } else { 0 };
            self.args.insert(name.into(), value.to_string());
        }
    }

    pub fn has_arg(&self, name: &str) -> bool {
        self.args.contains_key(name)
    }

    pub fn get_arg(&self, name: &str) -> Option<&String> {
        self.args.get(name)
    }

    pub fn args(&self) -> ArgIter<String, String> {
        self.args.iter()
    }

    pub fn add_file<N, P>(&mut self, name: N, file: P)
        where N: Into<String>, P: Into<PathBuf>
    {
        self.files.insert(name.into(), file.into());
    }

    pub fn get_file(&self, name: &str) -> Option<&PathBuf> {
        self.files.get(name)
    }

    pub fn files(&self) -> ArgIter<String, PathBuf> {
        self.files.iter()
    }
}

/// API connection.
pub trait APIConnection {
    fn api_url(&self) -> &String;

    fn send_get_request(&mut self,
                        url: &str,
                        args: &APIArguments) -> Result<APIResponse>;

    fn send_post_request(&mut self,
                         url: &str,
                         files: &APIArguments) -> Result<APIResponse>;
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
    fn api_url(&self) -> &String {
        self.conn.api_url()
    }

    fn send_get_request(&mut self,
                        url: &str,
                        args: &APIArguments) -> Result<APIResponse> {
        let response = self.conn.send_get_request(url, args)?;
        self.ensure_request_succeeded(&response)?;
        Ok(response)
    }

    fn send_post_request(&mut self,
                         url: &str,
                         args: &APIArguments) -> Result<APIResponse> {
        let response = self.conn.send_post_request(url, args)?;
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
                       args: &APIArguments) -> Result<HyperRequest<Fresh>> {
        let mut parsed_url = HyperUrl::parse(&url)
            .chain_err(|| "invalid API URL")?;
        for (key, value) in args.args() {
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
                    .map(|k| k.clone())
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
    fn api_url(&self) -> &String {
        self.settings.api_url()
    }

    fn send_get_request(&mut self,
                        url: &str,
                        args: &APIArguments) -> Result<APIResponse> {
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
                         args: &APIArguments) -> Result<APIResponse> {
        let request = self.prepare_request(HyperMethod::Post, &url, &args)
            .chain_err(|| format!("failed to prepare a POST request to {}", url))?;
        let mut mp = Multipart::new();
        for (name, file) in args.files() {
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

    use std::path::Path;

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

    #[test]
    fn api_arguments_add_string_arg_adds_string_argument() {
        let mut args = APIArguments::new();

        args.add_string_arg("name", "value");

        assert_eq!(args.get_arg("name"), Some(&"value".to_string()));
    }

    #[test]
    fn api_arguments_add_opt_string_arg_adds_string_argument_when_some() {
        let mut args = APIArguments::new();

        args.add_opt_string_arg("name", Some(&"value".to_string()));

        assert_eq!(args.get_arg("name"), Some(&"value".to_string()));
    }

    #[test]
    fn api_arguments_add_opt_string_arg_does_not_add_anything_when_none() {
        let mut args = APIArguments::new();

        args.add_opt_string_arg("name", None);

        assert!(!args.has_arg("name"));
    }

    #[test]
    fn api_arguments_add_bool_arg_adds_correct_arg_for_true() {
        let mut args = APIArguments::new();

        args.add_bool_arg("name", true);

        assert_eq!(args.get_arg("name"), Some(&"1".to_string()));
    }

    #[test]
    fn api_arguments_add_bool_arg_adds_correct_arg_for_false() {
        let mut args = APIArguments::new();

        args.add_bool_arg("name", false);

        assert_eq!(args.get_arg("name"), Some(&"0".to_string()));
    }

    #[test]
    fn api_arguments_add_opt_bool_arg_adds_bool_argument_when_some() {
        let mut args = APIArguments::new();

        args.add_opt_bool_arg("name", Some(true));

        assert_eq!(args.get_arg("name"), Some(&"1".to_string()));
    }

    #[test]
    fn api_arguments_add_opt_bool_arg_does_not_add_anything_when_none() {
        let mut args = APIArguments::new();

        args.add_opt_bool_arg("name", None);

        assert!(!args.has_arg("name"));
    }

    #[test]
    fn api_arguments_args_returns_iterator_over_arguments() {
        let mut args = APIArguments::new();

        args.add_string_arg("name", "value");

        let args: Vec<(&String, &String)> = args.args().collect();
        assert_eq!(args.len(), 1);
        assert_eq!(args[0].0, &"name".to_string());
        assert_eq!(args[0].1, &"value".to_string());
    }

    #[test]
    fn api_arguments_add_file_adds_file() {
        let mut args = APIArguments::new();

        args.add_file("input", "file.exe");

        assert_eq!(args.get_file("input"), Some(&Path::new("file.exe").to_path_buf()));
    }

    #[test]
    fn api_arguments_files_returns_iterator_over_files() {
        let mut args = APIArguments::new();

        args.add_file("input", "file.exe");

        let files: Vec<(&String, &PathBuf)> = args.files().collect();
        assert_eq!(files.len(), 1);
        assert_eq!(files[0].0, &"input".to_string());
        assert_eq!(files[0].1, &Path::new("file.exe").to_path_buf());
    }
}
