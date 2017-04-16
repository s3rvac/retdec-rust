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
use utils::current_platform_name;

/// Response from `retdec.com`'s API.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct APIResponse {
    status_code: u16,
    status_message: String,
    body: Vec<u8>,
}

impl APIResponse {
    /// Returns the status code (e.g. 200).
    pub fn status_code(&self) -> u16 {
        self.status_code
    }

    /// Returns the status message (e.g. `"Not Found"` for HTTP 404).
    pub fn status_message(&self) -> &String {
        &self.status_message
    }

    /// Has the request succeeded?
    pub fn succeeded(&self) -> bool {
        self.status_code >= 200 && self.status_code <= 299
    }

    /// Has the request failed?
    pub fn failed(&self) -> bool {
        !self.succeeded()
    }

    /// Returns the error message (if any).
    ///
    /// It is a short reason why the request failed.
    pub fn error_message(&self) -> Option<String> {
        self.json_value_as_string("message")
    }

    /// Returns the error description (if any).
    ///
    /// It is a longer reason why the request failed.
    pub fn error_description(&self) -> Option<String> {
        self.json_value_as_string("description")
    }

    /// Returns the body of the response as bytes.
    pub fn body(&self) -> &Vec<u8> {
        &self.body
    }

    /// Returns the body of the response as a non-owned UTF-8 string.
    pub fn body_as_str(&self) -> Result<&str> {
        str::from_utf8(&self.body)
            .chain_err(|| "failed to decode API response body as UTF-8")
    }

    /// Returns the body of the response as an owned UTF-8 string.
    pub fn body_as_string(&self) -> Result<String> {
        Ok(self.body_as_str()?.to_string())
    }

    /// Returns the body as a parsed JSON.
    pub fn body_as_json(&self) -> Result<JsonValue> {
        json::parse(self.body_as_str()?)
            .chain_err(|| "failed to parse API response body as JSON")
    }

    /// Returns the value of the given key in the parsed JSON body as a string.
    pub fn json_value_as_string(&self, key: &str) -> Option<String> {
        if let Ok(body) = self.body_as_json() {
            if let Some(value) = body[key].as_str() {
                return Some(value.to_string());
            }
        }
        None
    }

    /// Returns the value of the given key in the parsed JSON body as a bool.
    pub fn json_value_as_bool(&self, key: &str) -> Option<bool> {
        if let Ok(body) = self.body_as_json() {
            if let Some(value) = body[key].as_bool() {
                return Some(value);
            }
        }
        None
    }
}

/// Arguments passed to the `retdec.com`s API via GET/HTTP requests.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct APIArguments {
    args: HashMap<String, String>,
    files: HashMap<String, PathBuf>,
}

impl APIArguments {
    /// Creates empty arguments.
    pub fn new() -> Self {
        APIArguments::default()
    }

    /// Adds a new string argument (`"name=value"`).
    pub fn add_string_arg<N, V>(&mut self, name: N, value: V)
        where N: Into<String>,
              V: Into<String>
    {
        self.args.insert(name.into(), value.into());
    }

    /// Adds a new string argument (`"name=value"`) if `value` is not `None`.
    pub fn add_opt_string_arg<N>(&mut self, name: N, value: Option<&String>)
        where N: Into<String>
    {
        if let Some(value) = value {
            self.args.insert(name.into(), value.to_string());
        }
    }

    /// Adds a new bool argument (`"name=value"`).
    pub fn add_bool_arg<N>(&mut self, name: N, value: bool)
        where N: Into<String>
    {
        let value = if value { 1 } else { 0 };
        self.args.insert(name.into(), value.to_string());
    }

    /// Adds a new bool argument (`"name=value"`) if `value` is not `None`.
    pub fn add_opt_bool_arg<N>(&mut self, name: N, value: Option<bool>)
        where N: Into<String>
    {
        if let Some(value) = value {
            self.add_bool_arg(name, value);
        }
    }

    /// Is there an argument with the given name?
    pub fn has_arg(&self, name: &str) -> bool {
        self.args.contains_key(name)
    }

    /// Returns the argument with the given name.
    pub fn get_arg(&self, name: &str) -> Option<&String> {
        self.args.get(name)
    }

    /// Returns an iterator over arguments (`name` => `value`).
    pub fn args(&self) -> ArgIter<String, String> {
        self.args.iter()
    }

    /// Adds the given file under the given name.
    pub fn add_file<N, P>(&mut self, name: N, file: P)
        where N: Into<String>,
              P: Into<PathBuf>
    {
        self.files.insert(name.into(), file.into());
    }

    /// Returns a file with the given name.
    pub fn get_file(&self, name: &str) -> Option<&PathBuf> {
        self.files.get(name)
    }

    /// Returns an iterator over files (`name` => `path`).
    pub fn files(&self) -> ArgIter<String, PathBuf> {
        self.files.iter()
    }
}

/// A builder of API arguments.
#[cfg(test)]
#[derive(Debug, Default)]
pub struct APIArgumentsBuilder {
    args: APIArguments,
}

#[cfg(test)]
impl APIArgumentsBuilder {
    /// Creates a new builder.
    pub fn new() -> Self {
        APIArgumentsBuilder::default()
    }

    /// Adds a new string argument (`"name=value"`).
    pub fn with_string_arg<N, V>(mut self, name: N, value: V) -> Self
        where N: Into<String>,
              V: Into<String>
    {
        self.args.add_string_arg(name, value);
        self
    }

    /// Adds a new bool argument (`"name=value"`).
    pub fn with_bool_arg<N>(mut self, name: N, value: bool) -> Self
        where N: Into<String>
    {
        self.args.add_bool_arg(name, value);
        self
    }

    /// Adds a new file with the given name.
    pub fn with_file<N, P>(mut self, name: N, file: P) -> Self
        where N: Into<String>,
              P: Into<PathBuf>
    {
        self.args.add_file(name, file);
        self
    }

    /// Builds the arguments.
    pub fn build(self) -> APIArguments {
        self.args
    }
}

/// API connection.
pub trait APIConnection {
    /// Returns the URL to the API.
    fn api_url(&self) -> &String;

    /// Sends an HTTP GET request to the given url with the given arguments.
    fn send_get_request(&mut self,
                        url: &str,
                        args: &APIArguments) -> Result<APIResponse>;

    /// Sends an HTTP POST request to the given url with the given arguments.
    fn send_post_request(&mut self,
                         url: &str,
                         files: &APIArguments) -> Result<APIResponse>;

    /// Sends an HTTP GET request to the given url without any arguments.
    fn send_get_request_without_args(&mut self,
                                     url: &str) -> Result<APIResponse> {
        self.send_get_request(url, &APIArguments::new())
    }
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
    /// Creates a new connection to the API.
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
            .chain_err(|| "invalid URL")?;
        for (key, value) in args.args() {
            parsed_url.query_pairs_mut().append_pair(key, value);
        }
        let mut request = HyperRequest::<Fresh>::new(method, parsed_url)
            .chain_err(|| format!("failed to create a new request to {}", url))?;
        self.add_auth_to_request(&mut request)?;
        self.add_user_agent_to_request(&mut request);
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

    fn add_user_agent_to_request(&self, request: &mut HyperRequest<Fresh>) {
        let user_agent = format!("retdec-rust/{}", current_platform_name());
        request.headers_mut().set(header::UserAgent(user_agent));
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
            mp.add_file(
                name.clone(),
                file.to_str().ok_or(format!("cannot convert {:?} to str", file))?
            );
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
    /// Creates a new factory with the given settings.
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

/// Information about an API request.
#[cfg(test)]
#[derive(Debug, PartialEq)]
struct APIRequestInfo {
    method: &'static str,
    url: String,
    args: APIArguments,
}

/// Information about an API response.
#[cfg(test)]
#[derive(Debug)]
struct APIResponseInfo {
    method: &'static str,
    url: String,
    response: Result<APIResponse>,
}

/// A connection mock to be used in tests.
#[cfg(test)]
pub struct APIConnectionMock {
    settings: Settings,
    requests: Vec<APIRequestInfo>,
    responses: Vec<APIResponseInfo>,
}

#[cfg(test)]
impl APIConnectionMock {
    /// Creates a new mock with the given settings.
    pub fn new(settings: Settings) -> Self {
        APIConnectionMock {
            settings: settings,
            requests: Vec::new(),
            responses: Vec::new(),
        }
    }

    /// Adds a response to be returned for the given request.
    pub fn add_response<U>(&mut self,
                           method: &'static str,
                           url: U,
                           response: Result<APIResponse>)
        where U: Into<String>
    {
        self.responses.push(
            APIResponseInfo {
                method: method,
                url: url.into(),
                response: response,
            }
        );
    }

    /// Has a request to the given URL and with the given arguments been sent?
    pub fn request_sent<U>(&self,
                        method: &'static str,
                        url: U,
                        args: APIArguments) -> bool
        where U: Into<String>
    {
        self.requests.contains(
            &APIRequestInfo {
                method: method,
                url: url.into(),
                args: args
            }
        )
    }

    fn add_request<U>(&mut self,
                   method: &'static str,
                   url: U,
                   args: &APIArguments)
        where U: Into<String>
    {
        self.requests.push(
            APIRequestInfo {
                method: method,
                url: url.into(),
                args: (*args).clone(),
            }
        );
    }

    fn find_response(&mut self,
                     method: &'static str,
                     url: &str) -> Result<APIResponse> {
        let mut found_index = None;
        for (i, ref r) in self.responses.iter().enumerate() {
            if r.method == method && r.url == url {
                found_index = Some(i);
                break;
            }
        }
        if let Some(i) = found_index {
            return self.responses.remove(i).response;
        }
        panic!("no response set for {} request to {}", method, url);
    }
}

#[cfg(test)]
impl APIConnection for APIConnectionMock {
    fn api_url(&self) -> &String {
        self.settings.api_url()
    }

    fn send_get_request(&mut self,
                        url: &str,
                        args: &APIArguments) -> Result<APIResponse> {
        self.add_request("GET", url, args);
        self.find_response("GET", url)
    }

    fn send_post_request(&mut self,
                         url: &str,
                         args: &APIArguments) -> Result<APIResponse> {
        self.add_request("POST", url, args);
        self.find_response("POST", url)
    }
}

#[cfg(test)]
use std::rc::Rc;
#[cfg(test)]
use std::cell::RefCell;

#[cfg(test)]
struct InnerAPIConnectionMock {
    settings: Settings,
    conn: Rc<RefCell<APIConnectionMock>>,
}

#[cfg(test)]
impl APIConnection for InnerAPIConnectionMock {
    fn api_url(&self) -> &String {
        self.settings.api_url()
    }

    fn send_get_request(&mut self,
                        url: &str,
                        args: &APIArguments) -> Result<APIResponse> {
        self.conn.borrow_mut().send_get_request(url, args)
    }

    fn send_post_request(&mut self,
                         url: &str,
                         args: &APIArguments) -> Result<APIResponse> {
        self.conn.borrow_mut().send_post_request(url, args)
    }
}

/// A connection-factory mock to be used in tests.
#[cfg(test)]
pub struct APIConnectionFactoryMock {
    settings: Settings,
    conn: Rc<RefCell<APIConnectionMock>>,
}

#[cfg(test)]
impl APIConnectionFactoryMock {
    /// Creates a new factory.
    pub fn new(settings: Settings, conn: Rc<RefCell<APIConnectionMock>>) -> Self {
        APIConnectionFactoryMock {
            settings: settings,
            conn: conn,
        }
    }
}

#[cfg(test)]
impl APIConnectionFactory for APIConnectionFactoryMock {
    fn new_connection(&self) -> Box<APIConnection> {
        Box::new(
            InnerAPIConnectionMock {
                settings: self.settings.clone(),
                conn: self.conn.clone(),
            }
        )
    }
}

/// A builder of API responses.
#[cfg(test)]
#[derive(Debug, Default)]
pub struct APIResponseBuilder {
    response: APIResponse,
}

#[cfg(test)]
impl APIResponseBuilder {
    /// Creates a new builder.
    pub fn new() -> Self {
        APIResponseBuilder::default()
    }

    /// Sets the status code of the response.
    pub fn with_status_code(mut self, new_status_code: u16) -> Self {
        self.response.status_code = new_status_code;
        self
    }

    /// Sets the body of the response.
    pub fn with_body(mut self, new_body: &[u8]) -> Self {
        self.response.body = new_body.to_vec();
        self
    }

    /// Builds the response.
    pub fn build(self) -> APIResponse {
        self.response
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    use std::path::Path;

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
    fn api_response_failed_returns_true_when_response_failed() {
        let r = APIResponseBuilder::new()
            .with_status_code(404)
            .build();

        assert!(r.failed());
    }

    #[test]
    fn api_response_error_message_returns_error_message_when_present() {
        let r = APIResponseBuilder::new()
            .with_body(br#"{
                "message": "Unauthorized"
            }"#)
            .build();

        assert_eq!(r.error_message(), Some("Unauthorized".to_string()));
    }

    #[test]
    fn api_response_error_message_returns_none_when_no_error_message_present() {
        let r = APIResponseBuilder::new()
            .build();

        assert_eq!(r.error_message(), None);
    }

    #[test]
    fn api_response_error_description_returns_error_description_when_present() {
        let r = APIResponseBuilder::new()
            .with_body(br#"{
                "description": "API key authorization failed"
            }"#)
            .build();

        assert_eq!(r.error_description(), Some("API key authorization failed".to_string()));
    }

    #[test]
    fn api_response_error_description_returns_none_when_no_error_description_present() {
        let r = APIResponseBuilder::new()
            .build();

        assert_eq!(r.error_description(), None);
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
    fn api_response_json_value_as_string_returns_value_when_exists() {
        let r = APIResponseBuilder::new()
            .with_body(br#"{ "key": "value" }"#)
            .build();

        assert_eq!(r.json_value_as_string("key"), Some("value".to_string()));
    }

    #[test]
    fn api_response_json_value_as_string_returns_none_when_no_such_value() {
        let r = APIResponseBuilder::new()
            .build();

        assert_eq!(r.json_value_as_string("key"), None);
    }

    #[test]
    fn api_response_json_value_as_string_returns_none_when_value_has_different_type() {
        let r = APIResponseBuilder::new()
            .with_body(br#"{ "key": 1 }"#)
            .build();

        assert_eq!(r.json_value_as_string("key"), None);
    }

    #[test]
    fn api_response_json_value_as_bool_returns_value_when_exists() {
        let r = APIResponseBuilder::new()
            .with_body(br#"{ "key": true }"#)
            .build();

        assert_eq!(r.json_value_as_bool("key"), Some(true));
    }

    #[test]
    fn api_response_json_value_as_bool_returns_none_when_no_such_value() {
        let r = APIResponseBuilder::new()
            .build();

        assert_eq!(r.json_value_as_bool("key"), None);
    }

    #[test]
    fn api_response_json_value_as_bool_returns_none_when_value_has_different_type() {
        let r = APIResponseBuilder::new()
            .with_body(br#"{ "key": 1 }"#)
            .build();

        assert_eq!(r.json_value_as_bool("key"), None);
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
        assert_eq!(*args[0].0, "name".to_string());
        assert_eq!(*args[0].1, "value".to_string());
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
        assert_eq!(*files[0].0, "input".to_string());
        assert_eq!(*files[0].1, Path::new("file.exe").to_path_buf());
    }
}
