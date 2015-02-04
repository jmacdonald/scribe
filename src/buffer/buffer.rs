use std::old_io::{File, Open, ReadWrite};
use std::old_io::IoResult;
use super::GapBuffer;
use super::gap_buffer;
use super::Position;
use super::Range;

pub struct Buffer {
    data: GapBuffer,
    file: Option<File>,
    pub cursor: Position,
}

impl Buffer {
    /// Returns the contents of the buffer as a string.
    ///
    /// # Examples
    ///
    /// ```
    /// let buffer = scribe::buffer::from_file(&Path::new("tests/sample/file")).unwrap();
    /// assert_eq!(buffer.data(), "it works!\n");
    /// ```
    pub fn data(&self) -> String {
        self.data.to_string()
    }
}

/// Creates a new empty buffer. The buffer's cursor is set to the beginning of the buffer.
///
/// # Examples
///
/// ```
/// let buffer = scribe::buffer::new();
/// # assert_eq!(buffer.cursor.line, 0);
/// # assert_eq!(buffer.cursor.offset, 0);
/// ```
pub fn new() -> Buffer {
    let data = gap_buffer::new(String::new());
    let cursor = Position{ line: 0, offset: 0 };

    Buffer{ data: data, file: None, cursor: cursor }
}

/// Creates a new buffer by reading the UTF-8 interpreted file contents of the specified path.
/// The buffer's cursor is set to the beginning of the buffer.
///
/// # Examples
///
/// ```
/// let buffer = scribe::buffer::from_file(&Path::new("tests/sample/file")).unwrap();
/// assert_eq!(buffer.data(), "it works!\n");
/// # assert_eq!(buffer.cursor.line, 0);
/// # assert_eq!(buffer.cursor.offset, 0);
/// ```
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
    let cursor = Position{ line: 0, offset: 0 };

    // Create a new buffer using the loaded data, file, and other defaults.
    Ok(Buffer{ data: data, file: Some(file), cursor: cursor })
}
