extern crate luthor;

use std::rc::Rc;
use std::cell::RefCell;
use std::fs::File;
use std::io;
use std::io::{Read, Write};
use std::path::PathBuf;
use super::GapBuffer;
use super::gap_buffer;
use super::Position;
use super::Range;
use super::Cursor;
use super::type_detection;
use self::luthor::token::Token;
use self::luthor::lexers;

/// A UTF-8 buffer with bounds-checked cursor management and persistence.
pub struct Buffer {
    data: Rc<RefCell<GapBuffer>>,
    lexer: fn(&str) -> Vec<Token>,
    pub path: Option<PathBuf>,
    pub cursor: Cursor,
    data_cache: Option<String>,
    token_cache: Option<Vec<Token>>,
}

impl Buffer {
    /// Returns the contents of the buffer as a string. Caches
    /// this string representation to make subsequent requests
    /// to an unchanged buffer as fast as possible.
    ///
    /// # Examples
    ///
    /// ```
    /// let mut buffer = scribe::buffer::new();
    /// buffer.insert("scribe");
    /// assert_eq!(buffer.data(), "scribe");
    /// ```
    pub fn data(&mut self) -> String {
        match self.data_cache {
            Some(ref cache) => cache.clone(), // Cache hit; return a copy.
            None => {
                // Cache miss; update the cache w/ fresh data and return a copy.
                self.data_cache = Some(self.data.borrow().to_string());
                self.data_cache.clone().unwrap()
            }
        }
    }

    /// Writes the contents of the buffer to its path.
    ///
    /// # Examples
    ///
    /// ```
    /// # use std::path::PathBuf;
    /// # use std::path::Path;
    /// # use std::fs::File;
    /// # use std::io::Read;
    ///
    /// // Set up a buffer and point it to a path.
    /// let mut buffer = scribe::buffer::new();
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
    pub fn save(&mut self) -> Option<io::Error> {
        let path = match self.path.clone() {
            Some(p) => p,
            None => PathBuf::new(),
        };

        // Try to open and write to the file, returning any errors encountered.
        let mut file = match File::create(&path) {
            Ok(f) => f,
            Err(error) => return Some(error),
        };

        // We use to_string here because we don't want to write the gap contents.
        match file.write_all(self.data().to_string().as_bytes()) {
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
        // Insert the data at the cursor
        self.data.borrow_mut().insert(data, &self.cursor);

        // Caches are invalid as the buffer has changed.
        self.clear_caches();
    }

    /// Deletes a character at the cursor position. If at the end
    /// of the current line, it'll try to delete a newline character
    /// (joining the lines), succeeding if there's a line below.
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
        // We need to specify a range to delete, so start at
        // the current offset and delete the character to the right.
        let mut end = Position{ line: self.cursor.line, offset: self.cursor.offset + 1 };

        // If there isn't a character to the right,
        // delete the newline by jumping to the start
        // of the next line. If it doesn't exist, that's okay;
        // these values are bounds-checked by delete() anyway.
        if !self.data.borrow().in_bounds(&end) {
            end.line += 1;
            end.offset = 0;
        }

        // Delete the data.
        self.data.borrow_mut().delete(&Range{ start: *self.cursor, end: end});

        // Caches are invalid as the buffer has changed.
        self.clear_caches();
    }

    /// Produces a set of tokens based on the buffer data
    /// suitable for colorized display, using a lexer for the
    /// buffer data's language and/or format. Caches this
    /// lexed representation to make subsequent requests
    /// to an unchanged buffer as fast as possible.
    ///
    /// # Examples
    ///
    /// ```
    /// let mut buffer = scribe::buffer::new();
    /// buffer.insert("scribe data");
    /// 
    /// // Build the buffer data string back by combining its token lexemes.
    /// let mut data = String::new();
    /// for token in buffer.tokens().iter() {
    ///     data.push_str(&token.lexeme);
    /// }
    /// assert_eq!(data, "scribe data");
    /// ```
    pub fn tokens(&mut self) -> Vec<Token> {
        match self.token_cache {
            Some(ref cache) => cache.clone(), // Cache hit; return a copy.
            None => {
                // Cache miss; update the cache w/ fresh tokens and return a copy.
                self.token_cache = Some((self.lexer)(&self.data()));
                self.token_cache.clone().unwrap()
            }
        }
    }

    /// Returns the file name portion of the buffer's path, if
    /// the path is set and its file name is a valid UTF-8 sequence.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::path::PathBuf;
    ///
    /// let file_path = PathBuf::from("tests/sample/file");
    /// let buffer = scribe::buffer::from_file(file_path).unwrap();
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

    /// Called when caches are invalidated via buffer modifications.
    fn clear_caches(&mut self) {
        self.data_cache = None;
        self.token_cache = None;
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

    Buffer{
        data: data.clone(),
        path: None,
        cursor: cursor,
        lexer: lexers::default::lex as fn(&str) -> Vec<Token>,
        data_cache: None,
        token_cache: None,
    }
}

/// Creates a new buffer by reading the UTF-8 interpreted file contents of the specified path.
/// The buffer's cursor is set to the beginning of the buffer. The buffer data's type will be
/// inferred based on its extension, and an appropriate lexer will be used, if available (see
/// tokens method for further information on why this happens).
///
/// # Examples
///
/// ```
/// use std::path::PathBuf;
///
/// let file_path = PathBuf::from("tests/sample/file");
/// let mut buffer = scribe::buffer::from_file(file_path).unwrap();
/// assert_eq!(buffer.data(), "it works!\n");
/// # assert_eq!(buffer.cursor.line, 0);
/// # assert_eq!(buffer.cursor.offset, 0);
/// ```
pub fn from_file(path: PathBuf) -> io::Result<Buffer> {
    // Try to open and read the file, returning any errors encountered.
    let mut file = match File::open(path.clone()) {
        Ok(f) => f,
        Err(error) => return Err(error),
    };
    let mut data = String::new();
    match file.read_to_string(&mut data) {
        Ok(_) => (),
        Err(error) => return Err(error),
    };

    let data = Rc::new(RefCell::new(gap_buffer::new(data)));
    let cursor = Cursor{ data: data.clone(), position: Position{ line: 0, offset: 0 }};

    // Detect the file type and use its corresponding lexer, if available.
    let lexer = match type_detection::from_path(&path) {
        Some(type_detection::Type::JSON) => lexers::json::lex as fn(&str) -> Vec<Token>,
        Some(type_detection::Type::XML) => lexers::xml::lex as fn(&str) -> Vec<Token>,
        _ => lexers::default::lex as fn(&str) -> Vec<Token>,
    };

    // Create a new buffer using the loaded data, path, and other defaults.
    Ok(
        Buffer{
            data: data.clone(),
            path: Some(path),
            cursor: cursor,
            lexer: lexer,
            data_cache: None,
            token_cache: None,
        }
    )
}

#[cfg(test)]
mod tests {
    use super::new;
    use super::luthor::token::{Token, Category};

    #[test]
    fn tokens_returns_result_of_lexer() {
        let mut buffer = new();
        buffer.insert("scribe data");
        let expected_tokens = vec![
            Token{ lexeme: "scribe".to_string(), category: Category::Text },
            Token{ lexeme: " ".to_string(), category: Category::Whitespace },
            Token{ lexeme: "data".to_string(), category: Category::Text },
        ];
        assert_eq!(buffer.tokens(), expected_tokens);
    }

    #[test]
    fn delete_joins_lines_when_invoked_at_end_of_line() {
        let mut buffer = new();
        buffer.insert("scribe\n library");
        buffer.cursor.move_to_end_of_line();
        buffer.delete();
        assert_eq!(buffer.data(), "scribe library");
    }

    #[test]
    fn delete_does_nothing_when_invoked_at_the_end_of_the_document() {
        let mut buffer = new();
        buffer.insert("scribe\n library");
        buffer.cursor.move_down();
        buffer.cursor.move_to_end_of_line();
        buffer.delete();
        assert_eq!(buffer.data(), "scribe\n library");
    }

    #[test]
    fn insert_clears_data_and_token_caches() {
        let mut buffer = new();
        buffer.insert("scribe");

        // Trigger data and token cache storage.
        assert_eq!("scribe", buffer.data());
        assert_eq!(
            vec![Token{ lexeme: "scribe".to_string(), category: Category::Text }],
            buffer.tokens()
        );

        // Change the buffer contents using delete.
        buffer.insert("test_");

        // Ensure the cache has been busted.
        assert_eq!("test_scribe", buffer.data());
        assert_eq!(
            vec![Token{ lexeme: "test_scribe".to_string(), category: Category::Text }],
            buffer.tokens()
        );
    }

    #[test]
    fn delete_clears_data_and_token_caches() {
        let mut buffer = new();
        buffer.insert("scribe");

        // Trigger data and token cache storage.
        assert_eq!("scribe", buffer.data());
        assert_eq!(
            vec![Token{ lexeme: "scribe".to_string(), category: Category::Text }],
            buffer.tokens()
        );

        // Change the buffer contents using delete.
        buffer.delete();

        // Ensure the cache has been busted.
        assert_eq!("cribe", buffer.data());
        assert_eq!(
            vec![Token{ lexeme: "cribe".to_string(), category: Category::Text }],
            buffer.tokens()
        );
    }
}
