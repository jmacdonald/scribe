use std::old_io::{File, Open, ReadWrite};
use std::old_io::IoResult;
use super::GapBuffer;
use super::gap_buffer;
use super::Position;
use super::Range;

pub struct Buffer {
    data: GapBuffer,
    file: Option<File>,
}

impl Buffer {
    /// Returns the contents of the buffer as a string.
    ///
    /// # Examples
    ///
    /// ```rust
    ///
    /// let buffer = scribe::buffer::from_file(&Path::new("tests/sample/file")).unwrap();
    /// assert_eq!(buffer.data(), "it works!\n");
    pub fn data(&self) -> String {
        self.data.to_string()
    }
}

/// Creates a new buffer by reading the UTF-8 interpreted file contents of the specified path.
///
/// # Examples
///
/// ```rust
/// 
/// let buffer = scribe::buffer::from_file(&Path::new("tests/sample/file")).unwrap();
/// assert_eq!(buffer.data(), "it works!\n");
pub fn from_file(path: &Path) -> IoResult<Buffer> {
    // Try to open and read the file, returning any errors encountered.
    let mut file = match File::open_mode(path, Open, ReadWrite) {
        Ok(f) => f,
        Err(error) => return Err(error),
    };
    let mut data = match file.read_to_string() {
        Ok(d) => d,
        Err(error) => return Err(error),
    };

    let data = gap_buffer::new(data);

    // Create a new buffer using the loaded data, file, and other defaults.
    Ok(Buffer{ data: data, file: Some(file) })
}
