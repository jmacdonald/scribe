use std::cell::RefCell;
use std::old_io::{File, Open, Read, Write};
use std::old_io::IoError;
use std::old_io::IoResult;
use std::ops::Deref;
use super::GapBuffer;
use super::gap_buffer;
use super::Position;
use super::Range;

/// A UTF-8 buffer with bounds-checked cursor management and persistence.
pub struct Buffer {
    data: RefCell<GapBuffer>,
    pub path: Option<Path>,
    pub cursor: Cursor,
}

/// Read-only wrapper for a `Position`, to allow field level access to a
/// buffer's cursor while simultaneously enforcing bounds-checking when
/// updating its value.
pub struct Cursor {
    position: Position,
}

impl Deref for Cursor {
    type Target = Position;

    fn deref(&self) -> &Position {
        &self.position
    }
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

    /// Moves the buffer cursor to the specified location. The location is
    /// bounds-checked against the buffer data and the cursor will not be
    /// updated if it is out-of-bounds.
    ///
    /// # Examples
    ///
    /// ```
    /// let mut buffer = scribe::buffer::new();
    /// let in_bounds = scribe::buffer::Position{ line: 0, offset: 2 };
    /// let out_of_bounds = scribe::buffer::Position{ line: 2, offset: 2 };
    /// buffer.insert("scribe");
    ///
    /// buffer.move_cursor(in_bounds);
    /// assert_eq!(buffer.cursor.line, 0);
    /// assert_eq!(buffer.cursor.offset, 2);
    ///
    /// buffer.move_cursor(out_of_bounds);
    /// assert_eq!(buffer.cursor.line, 0);
    /// assert_eq!(buffer.cursor.offset, 2);
    /// ```
    pub fn move_cursor(&mut self, position: Position) {
        if self.data.borrow().in_bounds(&position) {
            self.cursor.position = position;
        }
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
    let data = RefCell::new(gap_buffer::new(String::new()));
    let cursor = Cursor{ position: Position{ line: 0, offset: 0 }};

    Buffer{ data: data, path: None, cursor: cursor }
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

    let data = RefCell::new(gap_buffer::new(data));
    let cursor = Cursor{ position: Position{ line: 0, offset: 0 }};

    // Create a new buffer using the loaded data, path, and other defaults.
    Ok(Buffer{ data: data, path: Some(path.clone()), cursor: cursor })
}
