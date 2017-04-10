//! Error handling.

use std::io::Write;

error_chain!{}

/// Prints the given error to the given stream.
///
/// # Panics
///
/// Panics if the error cannot be written into the stream.
pub fn print_error(err: &Error, stream: &mut Write) {
    let write_err_msg = "error writing to the given stream";

    writeln!(stream, "error: {}", err).expect(write_err_msg);

    for cause in err.iter().skip(1) {
        writeln!(stream, "  caused by: {}", cause).expect(write_err_msg);
    }

    // Run the program with `RUST_BACKTRACE=1` to show the backtrace.
    if let Some(backtrace) = err.backtrace() {
        writeln!(stream, "{:?}", backtrace).expect(write_err_msg);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use error::Error;
    use error::ErrorKind;

    #[test]
    fn print_error_prints_error_to_given_stream() {
        let err = Error::from_kind(ErrorKind::Msg("invalid key".to_string()));
        let mut stream = Vec::new();

        print_error(&err, &mut stream);

        assert_eq!(String::from_utf8_lossy(&stream), "error: invalid key\n");
    }

    #[test]
    fn print_error_includes_cause_when_present() {
        let err = Error::with_chain(
            Error::from_kind(ErrorKind::Msg("encoding error".to_string())),
            ErrorKind::Msg("invalid key".to_string())
        );
        let mut stream = Vec::new();

        print_error(&err, &mut stream);

        assert_eq!(
            String::from_utf8_lossy(&stream),
            "error: invalid key\n  caused by: encoding error\n"
        );
    }
}
