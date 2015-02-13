use std::rc::Rc;
use std::cell::RefCell;
use std::old_io::{File, Open, Read, Write};
use std::old_io::IoError;
use std::old_io::IoResult;
use super::GapBuffer;
use super::gap_buffer;
use super::Position;
use super::Range;
use super::Cursor;

/// A UTF-8 buffer with bounds-checked cursor management and persistence.
pub struct Buffer {
    data: Rc<RefCell<GapBuffer>>,
    pub path: Option<Path>,
    pub cursor: Cursor,
}

impl Buffer {
    /// Returns the contents of the buffer as a string.
    ///
    /// # Examples
    ///
    /// ```
    /// let mut buffer = scribe::buffer::new();
    /// buffer.insert("scribe");
    /// assert_eq!(buffer.data(), "scribe");
    /// ```
    pub fn data(&self) -> String {
        self.data.borrow().to_string()
    }

    /// Writes the contents of the buffer to its path.
    ///
    /// # Examples
    ///
    /// ```
    /// # use std::old_io::{File, Open, Read};
    ///
    /// // Set up a buffer and point it to a path.
    /// let mut buffer = scribe::buffer::new();
    /// let write_path = Path::new("my_doc");
    /// buffer.path = Some(write_path.clone());
    ///
    /// // Put some data into the buffer and save it.
    /// buffer.insert("scribe");
    /// buffer.save();
    ///
    /// # let saved_data = File::open_mode(&write_path, Open, Read)
    /// #   .unwrap().read_to_string().unwrap();
    /// # assert_eq!(saved_data, "scribe");
    ///
    /// # std::old_io::fs::unlink(&write_path);
    /// ```
    pub fn save(&self) -> Option<IoError> {
        let path = match self.path.clone() {
            Some(p) => p,
            None => Path::new(""),
        };

        // Try to open and write to the file, returning any errors encountered.
        let mut file = match File::open_mode(&path, Open, Write) {
            Ok(f) => f,
            Err(error) => return Some(error),
        };
        match file.write(self.data().as_bytes()) {
            Ok(_) => (),
            Err(error) => return Some(error),
        }

        return None
    }

    /// Inserts `data` into the buffer at the cursor position.
    ///
    /// # Examples
    ///
    /// ```
    /// let mut buffer = scribe::buffer::new();
    /// buffer.insert("scribe");
    /// assert_eq!(buffer.data(), "scribe");
    /// ```
    pub fn insert(&mut self, data: &str) {
        self.data.borrow_mut().insert(data, &self.cursor);
    }

    /// Deletes a character at the cursor position.
    ///
    /// # Examples
    ///
    /// ```
    /// let mut buffer = scribe::buffer::new();
    /// buffer.insert("scribe");
    /// buffer.delete();
    /// assert_eq!(buffer.data(), "cribe");
    /// ```
    pub fn delete(&mut self) {
        let mut end = self.cursor.clone();
        end.offset += 1;
        self.data.borrow_mut().delete(&Range{ start: *self.cursor, end: end});
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
    let data = Rc::new(RefCell::new(gap_buffer::new(String::new())));
    let cursor = Cursor{ data: data.clone(), position: Position{ line: 0, offset: 0 }};

    Buffer{ data: data.clone(), path: None, cursor: cursor }
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
    let mut file = match File::open_mode(path, Open, Read) {
        Ok(f) => f,
        Err(error) => return Err(error),
    };
    let mut data = match file.read_to_string() {
        Ok(d) => d,
        Err(error) => return Err(error),
    };

    let data = Rc::new(RefCell::new(gap_buffer::new(data)));
    let cursor = Cursor{ data: data.clone(), position: Position{ line: 0, offset: 0 }};

    // Create a new buffer using the loaded data, path, and other defaults.
    Ok(Buffer{ data: data.clone(), path: Some(path.clone()), cursor: cursor })
}
