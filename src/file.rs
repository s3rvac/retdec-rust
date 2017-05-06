//! Representation of files.

use std::fs;
use std::io::Read;
use std::io::Write;
use std::path::Path;
use std::path::PathBuf;
use std::str;

use unidecode::unidecode;

use error::Result;
use error::ResultExt;

/// In-memory representation of a file.
///
/// Only the name and content of a file are accessible. Path to the file is not
/// stored.
///
/// # Examples
///
/// ```no_run
/// use std::path::Path;
/// use retdec::file::File;
///
/// let file = File::from_path("tests/file.exe").unwrap();
///
/// assert_eq!(file.name(), "file.exe");
///
/// let saved_file_path = file.save_into("another_dir").unwrap();
/// assert_eq!(saved_file_path, Path::new("another_dir/file.exe"));
/// ```
#[derive(Clone, Debug)]
pub struct File {
    content: Vec<u8>,
    name: String,
}

impl File {
    /// Creates a file from the given path.
    ///
    /// The name is detected automatically from the path.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use retdec::file::File;
    ///
    /// let file = File::from_path("tests/file.exe").unwrap();
    ///
    /// assert_eq!(file.name(), "file.exe");
    /// ```
    pub fn from_path<P>(path: P) -> Result<File>
        where P: AsRef<Path>
    {
        let path = path.as_ref();
        Ok(File {
            content: Self::read_file(path)?,
            name: Self::get_file_name(path)?,
        })
    }

    /// Creates a file from the given path but gives it a custom name.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use retdec::file::File;
    ///
    /// let file = File::from_path_with_custom_name("tests/file.exe", "other.exe").unwrap();
    ///
    /// assert_eq!(file.name(), "other.exe");
    /// ```
    pub fn from_path_with_custom_name<P, N>(path: P, name: N) -> Result<File>
        where P: AsRef<Path>,
              N: Into<String>
    {
        let path = path.as_ref();
        Ok(File {
            content: Self::read_file(path)?,
            name: name.into(),
        })
    }

    /// Creates a file with the given content and name.
    ///
    /// # Examples
    ///
    /// ```
    /// use retdec::file::File;
    ///
    /// let file = File::from_content_with_name(b"content", "file.txt");
    ///
    /// assert_eq!(file.content(), b"content");
    /// assert_eq!(file.name(), "file.txt");
    /// ```
    pub fn from_content_with_name<N>(content: &[u8], name: N) -> File
        where N: Into<String>
    {
        File {
            content: content.to_vec(),
            name: name.into(),
        }
    }

    /// Returns the raw content of the file.
    ///
    /// # Examples
    ///
    /// ```
    /// use retdec::file::File;
    ///
    /// let file = File::from_content_with_name(b"content", "file.txt");
    ///
    /// assert_eq!(file.content(), b"content");
    /// ```
    pub fn content(&self) -> &[u8] {
        &self.content
    }

    /// Returns the content of the file as text.
    ///
    /// The content is expected to be encoded as UTF-8, which is the encoding
    /// that `retdec.com`'s API uses.
    ///
    /// # Examples
    ///
    /// ```
    /// use retdec::file::File;
    ///
    /// let file = File::from_content_with_name(b"content", "file.txt");
    ///
    /// assert_eq!(file.content_as_text().unwrap(), "content");
    /// ```
    pub fn content_as_text(&self) -> Result<&str> {
        str::from_utf8(&self.content)
            .chain_err(|| "failed to parse file content as UTF-8")
    }

    /// Returns the number of bytes in the content.
    ///
    /// # Examples
    ///
    /// ```
    /// use retdec::file::File;
    ///
    /// let file = File::from_content_with_name(b"content", "file.txt");
    ///
    /// assert_eq!(file.content_len(), 7);
    /// ```
    pub fn content_len(&self) -> usize {
        self.content.len()
    }

    /// Returns the name of the file.
    ///
    /// # Examples
    ///
    /// ```
    /// use retdec::file::File;
    ///
    /// let file = File::from_content_with_name(b"content", "file.txt");
    ///
    /// assert_eq!(file.name(), "file.txt");
    /// ```
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns a modified version of the file's name that can be passed to the
    /// retdec.com's API.
    ///
    /// # Examples
    ///
    /// ```
    /// use retdec::file::File;
    ///
    /// let file = File::from_content_with_name(b"content", "jalapeño.txt");
    ///
    /// assert_eq!(file.safe_name(), "jalapeno.txt");
    /// ```
    pub fn safe_name(&self) -> String {
        // There is a limitation in the retdec.com's API concerning file names.
        // More specifically, file names cannot contain non-ASCII characters.
        // https://retdec.com/api/docs/essential_information.html#files
        let safe_name = unidecode(&self.name);

        // Moreover, we replace special characters with an underscore.
        fn is_special(c: char) -> bool {
            // unidecode() produces an ASCII string, so the following cast to
            // u8 is safe.
            let c = c as u8;
            c < 32 || c > 127
        }
        safe_name.chars().map(|c| if is_special(c) { '_' } else { c }).collect()
    }

    /// Stores a copy of the file into the given directory.
    ///
    /// Returns a path to the saved file.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use std::path::Path;
    /// use retdec::file::File;
    ///
    /// let file = File::from_path("tests/file.exe").unwrap();
    ///
    /// let saved_file_path = file.save_into("another_dir").unwrap();
    /// assert_eq!(saved_file_path, Path::new("another_dir/file.exe"));
    /// ```
    pub fn save_into<P>(&self, dir: P) -> Result<PathBuf>
        where P: AsRef<Path>
    {
        self.save_into_under_name(dir, self.name())
    }

    /// Stores a copy of the file into the given directory under a custom name.
    ///
    /// Returns a path to the saved file.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use std::path::Path;
    /// use retdec::file::File;
    ///
    /// let file = File::from_path("tests/file.exe").unwrap();
    ///
    /// let saved_file_path = file.save_into_under_name("another_dir", "test.exe").unwrap();
    /// assert_eq!(saved_file_path, Path::new("another_dir/test.exe"));
    /// ```
    pub fn save_into_under_name<P>(&self, dir: P, name: &str) -> Result<PathBuf>
        where P: AsRef<Path>
    {
        let file_path = dir.as_ref().join(name);
        Self::write_file(self.content(), &file_path)?;
        Ok(file_path)
    }

    /// Stores a copy of the file into the given path.
    ///
    /// The path is expected to be a file path. If you want to store the file
    /// into a directory, use either `save_into()` or `save_into_under_name()`.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use retdec::file::File;
    ///
    /// let file = File::from_path("tests/file.exe").unwrap();
    ///
    /// file.save_as("another_dir/test.exe").unwrap();
    /// ```
    pub fn save_as<P>(&self, path: P) -> Result<()>
        where P: AsRef<Path>
    {
        Self::write_file(self.content(), path.as_ref())?;
        Ok(())
    }

    fn read_file(path: &Path) -> Result<Vec<u8>> {
        let mut file = fs::File::open(path)
            .chain_err(|| format!("failed to open {:?}", path))?;
        let mut content = Vec::new();
        file.read_to_end(&mut content)
            .chain_err(|| format!("failed to read {:?}", path))?;
        Ok(content)
    }

    fn write_file(content: &[u8], path: &Path) -> Result<()> {
        let mut file = fs::File::create(path)
            .chain_err(|| format!("failed to open {:?} for writing", path))?;
        file.write(content)
            .chain_err(|| format!("failed to write content into {:?}", path))?;
        Ok(())
    }

    fn get_file_name(path: &Path) -> Result<String> {
        let file_name = path.file_name()
            .ok_or_else(|| format!("no file name in {:?}", path))?;
        let file_name = file_name.to_string_lossy();
        Ok(file_name.into_owned())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Note: Tests for methods that work with the filesystem are in
    //       tests/file.rs.

    #[test]
    fn file_from_content_with_name_returns_correct_file() {
        let file = File::from_content_with_name(b"content", "file.txt");

        assert_eq!(file.content(), b"content");
        assert_eq!(file.name(), "file.txt");
    }

    #[test]
    fn file_safe_name_returns_name_with_only_ascii_characters() {
        let file = File::from_content_with_name(b"content", "jalapeño.txt");

        assert_eq!(file.safe_name(), "jalapeno.txt");
    }

    #[test]
    fn file_safe_name_replaces_special_characters_with_underscores() {
        let file = File::from_content_with_name(b"content", "a\nb");

        assert_eq!(file.safe_name(), "a_b");
    }

    #[test]
    fn file_safe_name_keeps_spaces() {
        let file = File::from_content_with_name(b"content", "a b");

        assert_eq!(file.safe_name(), "a b");
    }

    #[test]
    fn file_content_as_text_returns_str_when_content_is_text() {
        let file = File::from_content_with_name(b"content", "file.txt");

        let content = file.content_as_text()
            .expect("expected the content to be text");

        assert_eq!(content, "content");
    }

    #[test]
    fn file_content_as_text_returns_error_when_content_is_not_text() {
        let file = File::from_content_with_name(b"\xc3\x28", "file.txt");

        assert!(file.content_as_text().is_err());
    }

    #[test]
    fn file_content_len_returns_correct_value() {
        let file = File::from_content_with_name(b"123456", "file.txt");

        assert_eq!(file.content_len(), 6);
    }
}
