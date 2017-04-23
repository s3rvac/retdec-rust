//! Representation of files.

use std::fs;
use std::io::Read;
use std::path::Path;

use error::Result;
use error::ResultExt;

/// In-memory representation of a file.
#[derive(Clone, Debug, PartialEq)]
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

    /// Returns the content of the file.
    pub fn content(&self) -> &[u8] {
        &self.content
    }

    /// Returns the name of the file.
    pub fn name(&self) -> &str {
        &self.name
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
}
