//! Types related to in-memory buffers.

// Lexing library
extern crate luthor;

// Published API
pub use self::gap_buffer::GapBuffer;
pub use self::distance::Distance;

pub use self::position::Position;
pub use self::range::Range;
pub use self::line_range::LineRange;
pub use self::cursor::Cursor;
pub use self::token::{Lexeme, Token, TokenSet};
pub use syntect::parsing::{Scope, ScopeStack};

// Child modules
mod gap_buffer;
mod distance;
mod position;
mod range;
mod line_range;
mod cursor;
mod operation;
mod operations;
mod token;

// Buffer type implementation
use errors::*;
use std::rc::Rc;
use std::cell::RefCell;
use std::default::Default;
use std::fs::File;
use std::io;
use std::io::{Read, Write};
use std::mem;
use std::ops::Fn;
use std::path::{Path, PathBuf};
use self::operation::{Operation, OperationGroup};
use self::operation::history::History;
use syntect::parsing::{SyntaxReference, SyntaxSet};

/// A feature-rich wrapper around an underlying gap buffer.
///
/// The buffer type wraps an in-memory buffer, providing file I/O, a bounds-checked moveable
/// cursor, undo/redo history, simple type/format detection, and lexing (producing categorized
/// tokens suitable for syntax-highlighted display).
///
/// If the buffer is configured with a `change_callback`, it will be called with
/// a position whenever the buffer is modified; it's particularly useful for
/// cache invalidation.
///
/// Further, if `syntax_reference` is set, `syntax_set` _must_ be set as well.
pub struct Buffer {
    pub id: Option<usize>,
    data: Rc<RefCell<GapBuffer>>,
    pub path: Option<PathBuf>,
    pub cursor: Cursor,
    history: History,
    operation_group: Option<OperationGroup>,
    pub change_callback: Option<Box<dyn Fn(Position)>>,
    pub syntax_reference: Option<SyntaxReference>,
    pub syntax_set: Option<Rc<SyntaxSet>>,
}

impl Default for Buffer {
    fn default() -> Self {
        let data = Rc::new(RefCell::new(GapBuffer::new(String::new())));
        let cursor = Cursor::new(data.clone(), Position{ line: 0, offset: 0 });

        let mut history = History::new();
        history.mark();

        Buffer {
            id: None,
            data: data.clone(),
            path: None,
            cursor,
            history: History::new(),
            operation_group: None,
            change_callback: None,
            syntax_reference: None,
            syntax_set: None,
        }
    }
}

impl Buffer {
    /// Creates a new empty buffer. The buffer's cursor is set to the beginning of the buffer.
    ///
    /// # Examples
    ///
    /// ```
    /// use scribe::Buffer;
    ///
    /// let buffer = Buffer::new();
    /// # assert_eq!(buffer.cursor.line, 0);
    /// # assert_eq!(buffer.cursor.offset, 0);
    /// ```
    pub fn new() -> Buffer {
        Default::default()
    }

    /// Creates a new buffer by reading the UTF-8 interpreted file contents of the specified path.
    /// The buffer's cursor is set to the beginning of the buffer. The buffer data's type will be
    /// inferred based on its extension, and an appropriate lexer will be used, if available (see
    /// tokens method for further information on why this happens).
    /// The provided path is converted to its canonical, absolute equivalent,
    /// and stored alongside the buffer data.
    ///
    /// # Examples
    ///
    /// ```
    /// use scribe::Buffer;
    /// use std::path::Path;
    ///
    /// let file_path = Path::new("tests/sample/file");
    /// let mut buffer = Buffer::from_file(file_path).unwrap();
    /// assert_eq!(buffer.data(), "it works!\n");
    /// # assert_eq!(buffer.cursor.line, 0);
    /// # assert_eq!(buffer.cursor.offset, 0);
    /// ```
    pub fn from_file(path: &Path) -> io::Result<Buffer> {
        // Try to open and read the file, returning any errors encountered.
        let mut file = File::open(path)?;
        let mut data = String::new();
        file.read_to_string(&mut data)?;

        let data = Rc::new(RefCell::new(GapBuffer::new(data)));
        let cursor = Cursor::new(data.clone(), Position{ line: 0, offset: 0 });

        // Create a new buffer using the loaded data, path, and other defaults.
        let mut buffer =  Buffer{
            id: None,
            data: data.clone(),
            path: Some(path.canonicalize()?),
            cursor,
            ..Default::default()
        };

        // We mark the history at points where the
        // buffer is in sync with its file equivalent.
        buffer.history.mark();

        Ok(buffer)
    }

    /// Returns the contents of the buffer as a string.
    ///
    /// # Examples
    ///
    /// ```
    /// use scribe::Buffer;
    ///
    /// let mut buffer = Buffer::new();
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
    /// use scribe::Buffer;
    /// # use std::path::{Path, PathBuf};
    /// # use std::fs::File;
    /// # use std::io::Read;
    ///
    /// // Set up a buffer and point it to a path.
    /// let mut buffer = Buffer::new();
    /// let write_path = PathBuf::from("my_doc");
    /// buffer.path = Some(write_path.clone());
    ///
    /// // Put some data into the buffer and save it.
    /// buffer.insert("scribe");
    /// buffer.save();
    ///
    /// # let mut saved_data = String::new();
    /// # File::open(Path::new("my_doc")).unwrap().
    /// #   read_to_string(&mut saved_data).unwrap();
    /// # assert_eq!(saved_data, "scribe");
    ///
    /// # std::fs::remove_file(&write_path);
    /// ```
    pub fn save(&mut self) -> io::Result<()> {
        // Try to open and write to the file, returning any errors encountered.
        let mut file =
            if let Some(ref path) = self.path {
                File::create(&path)?
            } else {
                File::create(&PathBuf::new())?
            };

        // We use to_string here because we don't want to write the gap contents.
        file.write_all(self.data().to_string().as_bytes())?;

        // We mark the history at points where the
        // buffer is in sync with its file equivalent.
        self.history.mark();

        Ok(())
    }

    /// Produces a set of tokens based on the buffer data
    /// suitable for colorized display, using a lexer for the
    /// buffer data's language and/or format.
    pub fn tokens<'a>(&'a self) -> Result<TokenSet<'a>> {
        match (self.syntax_reference.as_ref(), self.syntax_set.as_ref()) {
            (Some(syntax_ref), Some(syntax_set)) =>
                Ok(TokenSet::new(self.data(), syntax_ref, syntax_set)),

            _ => Err(ErrorKind::MissingSyntaxDefinition)?,
        }
    }

    /// Returns the scope stack for the token at the cursor location.
    ///
    /// # Examples
    ///
    /// ```
    /// use scribe::Buffer;
    /// use scribe::buffer::{Position, Scope, ScopeStack};
    /// # use scribe::Workspace;
    /// # use std::path::PathBuf;
    /// # use std::env;
    ///
    /// // Set up a buffer with Rust source content and
    /// // move the cursor to something of interest.
    /// let mut buffer = Buffer::new();
    /// buffer.insert("struct Buffer");
    /// buffer.cursor.move_to(Position{ line: 0, offset: 7 });
    ///
    /// // Omitted code to set up workspace / buffer syntax definition.
    /// # let path = PathBuf::from("file.rs");
    /// # buffer.path = Some(path);
    /// # let mut workspace = Workspace::new(&env::current_dir().unwrap()).unwrap();
    /// # workspace.add_buffer(buffer);
    /// #
    /// assert_eq!(
    ///     workspace.current_buffer().unwrap().current_scope().unwrap(),
    ///     ScopeStack::from_vec(
    ///         vec![
    ///             Scope::new("source.rust").unwrap(),
    ///             Scope::new("meta.struct.rust").unwrap(),
    ///             Scope::new("entity.name.struct.rust").unwrap()
    ///         ]
    ///     )
    /// );
    /// ```
    pub fn current_scope(&self) -> Result<ScopeStack> {
        let mut scope = None;
        let tokens = self.tokens()?;

        for token in tokens.iter() {
            if let Token::Lexeme(lexeme) = token {
                if lexeme.position > *self.cursor {
                    break;
                }

                scope = Some(lexeme.scope);
            }
        }

        scope.ok_or_else(|| ErrorKind::MissingScope.into())
    }

    /// Returns the file name portion of the buffer's path, if
    /// the path is set and its file name is a valid UTF-8 sequence.
    ///
    /// # Examples
    ///
    /// ```
    /// use scribe::Buffer;
    /// use std::path::Path;
    ///
    /// let file_path = Path::new("tests/sample/file");
    /// let buffer = Buffer::from_file(file_path).unwrap();
    /// assert_eq!(buffer.file_name().unwrap(), "file");
    /// ```
    pub fn file_name(&self) -> Option<String> {
        match self.path {
            Some(ref path) => {
                match path.file_name() {
                    Some(file_name) => {
                        match file_name.to_str() {
                            Some(utf8_file_name) => Some(utf8_file_name.to_string()),
                            None => None,
                        }
                    },
                    None => None,
                }
            },
            None => None,
        }
    }


    /// Reverses the last modification to the buffer.
    ///
    /// # Examples
    ///
    /// ```
    /// use scribe::Buffer;
    /// use scribe::buffer::Position;
    ///
    /// let mut buffer = Buffer::new();
    /// // Run an initial insert operation.
    /// buffer.insert("scribe");
    /// buffer.cursor.move_to(Position{ line: 0, offset: 6});
    ///
    /// // Run a second insert operation.
    /// buffer.insert(" library");
    /// assert_eq!("scribe library", buffer.data());
    ///
    /// // Undo the second operation.
    /// buffer.undo();
    /// assert_eq!("scribe", buffer.data());
    ///
    /// // Undo the first operation.
    /// buffer.undo();
    /// assert_eq!("", buffer.data());
    /// ```
    pub fn undo(&mut self) {
        // Look for an operation to undo. First, check if there's an open, non-empty
        // operation group. If not, try taking the last operation from the buffer history.
        let operation: Option<Box<dyn Operation>> = match self.operation_group.take() {
            Some(group) => {
                if group.is_empty() {
                    self.history.previous()
                } else {
                    Some(Box::new(group))
                }
            }
            None => self.history.previous(),
        };

        // If we found an eligible operation, reverse it.
        if let Some(mut op) = operation {
            op.reverse(self);
        }
    }

    /// Re-applies the last undone modification to the buffer.
    ///
    /// # Examples
    ///
    /// ```
    /// use scribe::Buffer;
    ///
    /// let mut buffer = Buffer::new();
    /// buffer.insert("scribe");
    ///
    /// buffer.undo();
    /// assert_eq!("", buffer.data());
    ///
    /// buffer.redo();
    /// assert_eq!("scribe", buffer.data());
    /// ```
    pub fn redo(&mut self) {
        // Look for an operation to apply.
        if let Some(mut op) = self.history.next() {
            op.run(self);
        }
    }

    /// Tries to read the specified range from the buffer.
    ///
    /// # Examples
    ///
    /// ```
    /// use scribe::Buffer;
    /// use scribe::buffer::{Position, Range};
    ///
    /// let mut buffer = Buffer::new();
    /// buffer.insert("scribe");
    ///
    /// let range = Range::new(
    ///     Position{ line: 0, offset: 1 },
    ///     Position{ line: 0, offset: 5 }
    /// );
    /// assert_eq!("crib", buffer.read(&range).unwrap());
    /// ```
    pub fn read(&self, range: &Range) -> Option<String> {
        self.data.borrow().read(range)
    }

    /// Searches the buffer for (and returns positions
    /// associated with) occurrences of `needle`.
    ///
    /// # Examples
    ///
    /// ```
    /// use scribe::Buffer;
    /// use scribe::buffer::Position;
    ///
    /// let mut buffer = Buffer::new();
    /// buffer.insert("scribe\nlibrary");
    ///
    /// assert_eq!(
    ///     buffer.search("ib"),
    ///     vec![
    ///         Position{ line: 0, offset: 3 },
    ///         Position{ line: 1, offset: 1 }
    ///     ]
    /// );
    /// ```
    pub fn search(&self, needle: &str) -> Vec<Position> {
        let mut results = Vec::new();

        for (line, data) in self.data().lines().enumerate() {
            for (offset, _) in data.char_indices() {
                let haystack = &data[offset..];

                // Check haystack length before slicing it and comparing bytes with needle.
                if haystack.len() >= needle.len() && needle.as_bytes() == &haystack.as_bytes()[..needle.len()] {
                    results.push(
                        Position{
                            line,
                            offset
                        }
                    );
                }
            }
        }

        results
    }

    /// Whether or not the buffer has been modified since being read from or
    /// written to disk. Buffers without paths are always considered modified.
    ///
    /// # Examples
    ///
    /// ```
    /// use scribe::Buffer;
    /// use std::path::Path;
    ///
    /// let file_path = Path::new("tests/sample/file");
    /// let mut buffer = Buffer::from_file(file_path).unwrap();
    ///
    /// assert!(!buffer.modified());
    ///
    /// // Inserting data into a buffer will flag it as modified.
    /// buffer.insert("scribe");
    /// assert!(buffer.modified());
    ///
    /// // Undoing the modification reverses the flag.
    /// buffer.undo();
    /// assert!(!buffer.modified());
    ///
    /// // Buffers without paths are always modified.
    /// buffer = Buffer::new();
    /// assert!(buffer.modified());
    /// ```
    pub fn modified(&self) -> bool {
        !self.history.at_mark()
    }

    /// The number of lines in the buffer, including trailing newlines.
    ///
    /// # Examples
    ///
    /// ```
    /// use scribe::Buffer;
    ///
    /// let mut buffer = Buffer::new();
    /// buffer.insert("scribe\nlibrary\n");
    ///
    /// assert_eq!(buffer.line_count(), 3);
    /// ```
    pub fn line_count(&self) -> usize {
        self.data().chars().filter(|&c| c == '\n').count() + 1
    }

    /// Reloads the buffer from disk, discarding any in-memory modifications and
    /// history, as well as resetting the cursor to its initial (0,0) position.
    /// The buffer's ID and syntax definition are persisted.
    ///
    /// # Examples
    ///
    /// ```
    /// use scribe::buffer::{Buffer, Position};
    /// use std::path::Path;
    ///
    /// let file_path = Path::new("tests/sample/file");
    /// let mut buffer = Buffer::from_file(file_path).unwrap();
    /// buffer.insert("scribe\nlibrary\n");
    /// buffer.reload();
    ///
    /// assert_eq!(buffer.data(), "it works!\n");
    /// assert_eq!(*buffer.cursor, Position{ line: 0, offset: 0 });
    /// # buffer.undo();
    /// # assert_eq!(buffer.data(), "it works!\n");
    /// ```
    pub fn reload(&mut self) -> io::Result<()> {
        if let Some(ref path) = self.path.clone() {
            match Buffer::from_file(path) {
                Ok(mut buf) => {
                    mem::swap(self, &mut buf);

                    // Restore the buffer's ID.
                    self.id = buf.id;
                    self.syntax_reference = buf.syntax_reference;
                    self.change_callback = buf.change_callback;
                },
                Err(e) => return Err(e),
            }
        }

        // Run the change callback, if present.
        if let Some(ref callback) = self.change_callback {
            callback(Position::new())
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    extern crate syntect;
    use syntect::parsing::SyntaxSet;
    use std::cell::RefCell;
    use std::path::Path;
    use std::rc::Rc;
    use buffer::{Buffer, Position};

    #[test]
    fn reload_persists_id_and_syntax_reference() {
        let file_path = Path::new("tests/sample/file");
        let mut buffer = Buffer::from_file(file_path).unwrap();

        // Load syntax higlighting.
        let syntax_set = SyntaxSet::load_defaults_newlines();
        let syntax_reference = Some(syntax_set.find_syntax_plain_text().clone());

        // Set the attributes we want to verify are persisted.
        buffer.id = Some(1);
        buffer.syntax_reference = syntax_reference;

        buffer.reload().unwrap();

        assert_eq!(buffer.id, Some(1));
        assert!(buffer.syntax_reference.is_some());
    }

    #[test]
    fn reload_calls_change_callback_with_zero_position() {
        // Load a buffer with some data and modify it.
        let file_path = Path::new("tests/sample/file");
        let mut buffer = Buffer::from_file(file_path).unwrap();
        buffer.insert("amp\neditor");

        // Create a non-zero position that we'll share with the callback.
        let tracked_position = Rc::new(RefCell::new(Position{ line: 1, offset: 1 }));
        let callback_position = tracked_position.clone();

        // Set up the callback so that it updates the shared position.
        buffer.change_callback = Some(Box::new(move |change_position| {
            *callback_position.borrow_mut() = change_position
        }));

        // Reload the buffer
        buffer.reload().unwrap();

        // Verify that the callback received the correct position.
        assert_eq!(*tracked_position.borrow(), Position::new());
    }

    #[test]
    fn delete_joins_lines_when_invoked_at_end_of_line() {
        let mut buffer = Buffer::new();
        buffer.insert("scribe\n library");
        buffer.cursor.move_to_end_of_line();
        buffer.delete();
        assert_eq!(buffer.data(), "scribe library");
    }

    #[test]
    fn delete_does_nothing_when_invoked_at_the_end_of_the_document() {
        let mut buffer = Buffer::new();
        buffer.insert("scribe\n library");
        buffer.cursor.move_down();
        buffer.cursor.move_to_end_of_line();
        buffer.delete();
        assert_eq!(buffer.data(), "scribe\n library");
    }

    #[test]
    fn insert_is_undoable() {
        let mut buffer = Buffer::new();
        buffer.insert("scribe");
        assert_eq!("scribe", buffer.data());
        buffer.undo();
        assert_eq!("", buffer.data());
    }

    #[test]
    fn delete_is_undoable() {
        let mut buffer = Buffer::new();
        buffer.insert("scribe");
        assert_eq!("scribe", buffer.data());

        buffer.cursor.move_to(Position{ line: 0, offset: 0 });
        buffer.delete();
        assert_eq!("cribe", buffer.data());

        buffer.undo();
        assert_eq!("scribe", buffer.data());
    }

    #[test]
    fn correctly_called_operation_groups_are_undone_correctly() {
        let mut buffer = Buffer::new();

        // Run some operations in a group.
        buffer.start_operation_group();
        buffer.insert("scribe");
        buffer.cursor.move_to(Position{ line: 0, offset: 6});
        buffer.insert(" library");
        buffer.end_operation_group();

        // Run an operation outside of the group.
        buffer.cursor.move_to(Position{ line: 0, offset: 14});
        buffer.insert(" test");

        // Make sure the buffer looks okay.
        assert_eq!("scribe library test", buffer.data());

        // Check that undo reverses the single operation outside the group.
        buffer.undo();
        assert_eq!("scribe library", buffer.data());

        // Check that undo reverses the group operation.
        buffer.undo();
        assert_eq!("", buffer.data());
    }

    #[test]
    fn non_terminated_operation_groups_are_undone_correctly() {
        let mut buffer = Buffer::new();

        // Run an operation outside of the group.
        buffer.insert("scribe");

        // Run some operations in a group, without closing it.
        buffer.start_operation_group();
        buffer.cursor.move_to(Position{ line: 0, offset: 6});
        buffer.insert(" library");
        buffer.cursor.move_to(Position{ line: 0, offset: 14});
        buffer.insert(" test");

        // Make sure the buffer looks okay.
        assert_eq!("scribe library test", buffer.data());

        // Check that undo reverses the single operation outside the group.
        buffer.undo();
        assert_eq!("scribe", buffer.data());

        // Check that undo reverses the group operation.
        buffer.undo();
        assert_eq!("", buffer.data());
    }

    #[test]
    fn non_terminated_empty_operation_groups_are_dropped() {
        let mut buffer = Buffer::new();

        // Run an operation outside of the group.
        buffer.insert("scribe");

        // Start an empty operation group.
        buffer.start_operation_group();

        // Check that undo drops the empty operation group
        // and undoes the previous operation.
        buffer.undo();
        assert_eq!(buffer.data(), "");
    }

    #[test]
    fn search_returns_empty_set_when_there_are_no_matches() {
        let mut buffer = Buffer::new();

        // Run an operation outside of the group.
        buffer.insert("scribe");

        assert!(buffer.search("library").is_empty());
    }

    #[test]
    fn search_does_not_panic_with_non_ascii_data() {
        let mut buffer = Buffer::new();

        // Run an operation outside of the group.
        buffer.insert("scribé");

        // Use a longer term than the haystack.
        assert!(buffer.search("library").is_empty());

        // Use a term whose length does not lie on a haystack character boundary.
        assert!(buffer.search("scribe").is_empty());

        // Use a matching term.
        assert!(buffer.search("scribé").len() > 0);
    }
}
