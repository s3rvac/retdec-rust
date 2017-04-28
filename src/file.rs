//! Representation of files.

use std::fs;
use std::io::Read;
use std::io::Write;
use std::path::Path;
use std::path::PathBuf;
use std::str;

use error::Result;
use error::ResultExt;

/// In-memory representation of a file.
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
            content: Self::read_file(&path)?,
            name: Self::get_file_name(&path)?,
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
        where P: AsRef<Path>, N: Into<String>
    {
        let path = path.as_ref();
        Ok(File {
            content: Self::read_file(&path)?,
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
            name: name.into()
        }
    }

    /// Returns the raw content of the file.
    pub fn content(&self) -> &[u8] {
        &self.content
    }

    /// Returns the content of the file as text.
    ///
    /// The content is expected to be encoded as UTF-8, which is the encoding
    /// that `retdec.com`'s API uses.
    pub fn content_as_text(&self) -> Result<&str> {
        str::from_utf8(&self.content)
            .chain_err(|| "failed to parse file content as UTF-8")
    }

    /// Returns the number of bytes in the content.
    pub fn content_len(&self) -> usize {
        self.content.len()
    }

    /// Returns the name of the file.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Stores a copy of the file into the given directory.
    ///
    /// Returns a path to the saved file.
    pub fn save_into<P>(&self, dir: P) -> Result<PathBuf>
        where P: AsRef<Path>
    {
        let file_path = dir.as_ref().join(self.name());
        let mut file = fs::File::create(&file_path)
            .chain_err(|| format!("failed to open {:?} for writing", file_path))?;
        file.write(self.content())
            .chain_err(|| format!("failed to write content into {:?}", file_path))?;
        Ok(file_path)
    }

    /// Stores a copy of the file into the given directory under a custom name.
    ///
    /// Returns a path to the saved file.
    pub fn save_into_under_name<P>(&self, dir: P, name: &str) -> Result<PathBuf>
        where P: AsRef<Path>
    {
        let file_path = dir.as_ref().join(name);
        let mut file = fs::File::create(&file_path)
            .chain_err(|| format!("failed to open {:?} for writing", file_path))?;
        file.write(self.content())
            .chain_err(|| format!("failed to write content into {:?}", file_path))?;
        Ok(file_path)
    }

    fn read_file(path: &Path) -> Result<Vec<u8>> {
        let mut file = fs::File::open(path)
            .chain_err(|| format!("failed to open {:?}", path))?;
        let mut content = Vec::new();
        file.read_to_end(&mut content)
            .chain_err(|| format!("failed to read {:?}", path))?;
        Ok(content)
    }

    fn get_file_name(path: &Path) -> Result<String> {
        let file_name = path.file_name()
            .ok_or(format!("no file name in {:?}", path))?;
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
