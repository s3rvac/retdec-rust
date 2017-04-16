//! Access to the file-decompiling service
//! ([decompiler](https://retdec.com/api/docs/decompiler.html)).

use connection::APIArguments;
use connection::APIConnectionFactory;
use connection::HyperAPIConnectionFactory;
use decompilation::Decompilation;
use decompilation::DecompilationArguments;
use error::Result;
use error::ResultExt;
use settings::Settings;

/// File-decompiling service.
///
/// # Examples
///
/// ```no_run
/// use std::path::Path;
///
/// use retdec::decompilation::DecompilationArguments;
/// use retdec::decompiler::Decompiler;
/// use retdec::settings::Settings;
///
/// let settings = Settings::new()
///     .with_api_key("MY-API-KEY");
/// let decompiler = Decompiler::new(settings);
/// let args = DecompilationArguments::new()
///     .with_input_file(Path::new(&"file.exe").to_path_buf());
/// let mut decompilation = decompiler.start_decompilation(&args).unwrap();
/// decompilation.wait_until_finished().unwrap();
/// let output_hll = decompilation.get_output_hll().unwrap();
/// print!("{}", output_hll);
/// ```
pub struct Decompiler {
    conn_factory: Box<APIConnectionFactory>,
}

impl Decompiler {
    /// Creates a new instance of the file-decompiling service.
    pub fn new(settings: Settings) -> Self {
        Decompiler {
            conn_factory: Box::new(HyperAPIConnectionFactory::new(settings)),
        }
    }

    /// Starts a new decompilation with the given arguments.
    pub fn start_decompilation(&self, args: &DecompilationArguments) -> Result<Decompilation> {
        let mut conn = self.conn_factory.new_connection();
        let url = format!("{}/decompiler/decompilations", conn.api_url());
        let api_args = self.create_api_args(args)?;
        let response = conn.send_post_request(&url, &api_args)
            .chain_err(|| "failed to start a decompilation")?;
        let id = response.json_value_as_string("id")
            .ok_or(format!("{} returned invalid JSON response", url))?;
        Ok(Decompilation::new(id, conn))
    }

    fn create_api_args(&self, args: &DecompilationArguments) -> Result<APIArguments> {
        let mut api_args = APIArguments::new();
        api_args.add_string_arg("mode", "bin");
        match args.input_file() {
            Some(ref input_file) => {
                api_args.add_file("input", input_file);
            }
            None => {
                bail!("no input file given");
            }
        }
        Ok(api_args)
    }

    #[cfg(test)]
    fn with_conn_factory(conn_factory: Box<APIConnectionFactory>) -> Self {
        Decompiler { conn_factory: conn_factory }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::cell::RefCell;
    use std::path::Path;
    use std::rc::Rc;

    use decompilation::DecompilationArguments;
    use connection::APIArgumentsBuilder;
    use connection::APIConnectionFactoryMock;
    use connection::APIConnectionMock;
    use connection::APIResponseBuilder;

    fn create_decompiler(settings: Settings) -> (Rc<RefCell<APIConnectionMock>>, Decompiler) {
        let conn = Rc::new(
            RefCell::new(
                APIConnectionMock::new(settings.clone())
            )
        );
        let conn_factory = Box::new(
            APIConnectionFactoryMock::new(
                settings.clone(),
                conn.clone()
            )
        );
        (conn, Decompiler::with_conn_factory(conn_factory))
    }

    #[test]
    fn decompiler_start_decompilation_starts_decompilation_with_correct_arguments() {
        let (conn, decompiler) = create_decompiler(Settings::new());
        let input_file = Path::new(&"file.exe").to_path_buf();
        let args = DecompilationArguments::new()
            .with_input_file(input_file.clone());
        conn.borrow_mut().add_response(
            "POST",
            "https://retdec.com/service/api/decompiler/decompilations",
            Ok(
                APIResponseBuilder::new()
                    .with_status_code(200)
                    .with_body(br#"{
                        "id": "ID"
                    }"#)
                    .build()
            )
        );

        let decompilation = decompiler.start_decompilation(&args)
            .expect("decompilation should have succeeded");

        assert_eq!(*decompilation.id(), "ID");
        assert!(conn.borrow_mut().request_sent(
            "POST",
            "https://retdec.com/service/api/decompiler/decompilations",
            APIArgumentsBuilder::new()
                .with_string_arg("mode", "bin")
                .with_file("input", input_file.clone())
                .build()
        ));
    }

    #[test]
    fn decompiler_start_decompilation_returns_error_when_input_file_is_not_given() {
        let (conn, decompiler) = create_decompiler(Settings::new());
        let args = DecompilationArguments::new();
        conn.borrow_mut().add_response(
            "POST",
            "https://retdec.com/service/api/decompiler/decompilations",
            Ok(
                APIResponseBuilder::new()
                    .with_status_code(200)
                    .with_body(br#"{
                        "id": "ID"
                    }"#)
                    .build()
            )
        );

        let result = decompiler.start_decompilation(&args);

        let err = result.err().expect("expected start_decompilation() to fail");
        assert_eq!(err.description(), "no input file given");
    }

    #[test]
    fn decompiler_start_decompilation_returns_error_when_returned_json_does_not_contain_id() {
        let (conn, decompiler) = create_decompiler(Settings::new());
        let input_file = Path::new(&"file.exe").to_path_buf();
        let args = DecompilationArguments::new()
            .with_input_file(input_file.clone());
        conn.borrow_mut().add_response(
            "POST",
            "https://retdec.com/service/api/decompiler/decompilations",
            Ok(
                APIResponseBuilder::new()
                    .with_status_code(200)
                    .with_body(b"{}")
                    .build()
            )
        );

        let result = decompiler.start_decompilation(&args);

        let err = result.err().expect("expected start_decompilation() to fail");
        assert_eq!(
            err.description(),
            "https://retdec.com/service/api/decompiler/decompilations returned invalid JSON response"
        );
    }
}
