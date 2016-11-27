//! Buffer and working directory management.

use buffer::Buffer;
use std::io;
use std::path::{Path, PathBuf};
use syntect::parsing::{SyntaxDefinition, SyntaxSet};

/// An owned collection of buffers and associated path,
/// representing a running editor environment.
pub struct Workspace {
    pub path: PathBuf,
    buffers: Vec<Buffer>,
    next_buffer_id: usize,
    current_buffer_index: Option<usize>,
    syntax_set: SyntaxSet,
}

impl Workspace {
    /// Creates a new empty workspace for the specified path.
    pub fn new(path: &Path) -> io::Result<Workspace> {
        // Set up syntax parsers.
        let mut syntax_set = SyntaxSet::load_defaults_newlines();
        syntax_set.link_syntaxes();

        Ok(Workspace{
            path: try!(path.canonicalize()),
            buffers: Vec::new(),
            next_buffer_id: 0,
            current_buffer_index: None,
            syntax_set: syntax_set,
        })
    }

    /// Adds a buffer to the workspace, *inserting it after the
    /// current buffer*, populates its `id` field with a unique
    /// value (relative to the workspace), and selects it.
    ///
    /// # Examples
    ///
    /// ```
    /// use scribe::Buffer;
    /// use scribe::Workspace;
    /// use std::path::Path;
    ///
    /// // Set up the paths we'll use.
    /// let directory_path = Path::new("tests/sample");
    /// let file_path = Path::new("tests/sample/file");
    ///
    /// // Create a workspace.
    /// let mut workspace = Workspace::new(directory_path).unwrap();
    ///
    /// // Add a buffer to the workspace.
    /// let buf = Buffer::from_file(file_path).unwrap();
    /// workspace.add_buffer(buf);
    /// ```
    pub fn add_buffer(&mut self, mut buf: Buffer) {
        // Set a unique buffer ID.
        buf.id = Some(self.next_buffer_id);

        // Increment the ID for the next time.
        self.next_buffer_id += 1;

        // The target index is directly after the current buffer's index.
        let target_index = self.current_buffer_index.map(|i| i + 1 ).unwrap_or(0);

        // Add a syntax definition to the buffer, if it doesn't already have one.
        if buf.syntax_definition.is_none() {
            buf.syntax_definition = self.find_syntax_definition(&buf);
        }

        // Insert the buffer and select it.
        self.buffers.insert(target_index, buf);
        self.current_buffer_index = Some(target_index);
    }

    /// Opens a buffer at the specified path, *inserting
    /// it after the current buffer*, and selects it.
    /// If a buffer with the specified path already exists,
    /// it is selected, rather than opening a duplicate buffer.
    /// Any errors encountered while opening the buffer are returned.
    ///
    /// # Examples
    ///
    /// ```
    /// use scribe::Workspace;
    /// use std::path::Path;
    ///
    /// // Set up the paths we'll use.
    /// let directory_path = Path::new("tests/sample");
    /// let file_path = Path::new("tests/sample/file");
    ///
    /// // Create a workspace.
    /// let mut workspace = Workspace::new(directory_path).unwrap();
    ///
    /// // Open a buffer in the workspace.
    /// workspace.open_buffer(file_path.clone());
    /// ```
    pub fn open_buffer(&mut self, path: &Path) -> io::Result<()> {
        if self.contains_buffer_with_path(path) {
            // We already have this buffer in the workspace.
            // Loop through the buffers until it's selected.
            let canonical_path = path.canonicalize().unwrap();
            loop {
                match self.current_buffer() {
                    Some(buffer) => {
                        match buffer.path {
                            Some(ref current_path) => {
                                if *current_path == canonical_path {
                                    break;
                                }
                            },
                            None => (),
                        }
                    },
                    None => (),
                }

                self.next_buffer()
            }

            // Not going to run into IO errors if we're not opening a buffer.
            Ok(())
        } else {
            let buffer = try!(Buffer::from_file(path));
            Ok(self.add_buffer(buffer))
        }
    }

    /// Returns a mutable reference to the currently
    /// selected buffer, unless the workspace is empty.
    ///
    /// # Examples
    ///
    /// ```
    /// use scribe::Buffer;
    /// use scribe::Workspace;
    /// use std::path::Path;
    ///
    /// // Set up the paths we'll use.
    /// let directory_path = Path::new("tests/sample");
    /// let file_path = Path::new("tests/sample/file");
    ///
    /// // Create a workspace.
    /// let mut workspace = Workspace::new(directory_path).unwrap();
    ///
    /// // Add a buffer to the workspace.
    /// let buf = Buffer::from_file(file_path).unwrap();
    /// workspace.add_buffer(buf);
    ///
    /// // Get a reference to the current buffer.
    /// let buffer_reference = workspace.current_buffer().unwrap();
    /// ```
    pub fn current_buffer(&mut self) -> Option<&mut Buffer> {
        match self.current_buffer_index {
            Some(index) => Some(&mut self.buffers[index]),
            None => None,
        }
    }

    /// Removes the currently selected buffer from the collection.
    /// If the workspace is empty, this method does nothing.
    ///
    /// # Examples
    ///
    /// ```
    /// use scribe::Buffer;
    /// use scribe::Workspace;
    /// use std::path::Path;
    ///
    /// // Set up the paths we'll use.
    /// let directory_path = Path::new("tests/sample");
    /// let file_path = Path::new("tests/sample/file");
    ///
    /// // Create a workspace.
    /// let mut workspace = Workspace::new(directory_path).unwrap();
    ///
    /// // Add a buffer to the workspace.
    /// let buf = Buffer::from_file(file_path).unwrap();
    /// workspace.add_buffer(buf);
    ///
    /// // Close the current buffer.
    /// workspace.close_current_buffer();
    /// ```
    pub fn close_current_buffer(&mut self) {
        if let Some(index) = self.current_buffer_index {
            self.buffers.remove(index);

            if self.buffers.is_empty() {
                self.current_buffer_index = None;
            } else {
                self.current_buffer_index = index.checked_sub(1).or(Some(0));
            }
        };
    }

    /// Selects the previous buffer in the workspace (buffers are ordered as
    /// they are added to the workspace). If the currently selected buffer is
    /// the first in the collection, this will wrap and select the last buffer.
    ///
    /// # Examples
    ///
    /// ```
    /// use scribe::Buffer;
    /// use scribe::Workspace;
    /// use std::path::Path;
    ///
    /// // Set up the paths we'll use.
    /// let directory_path = Path::new("tests/sample");
    /// let file_path = Path::new("tests/sample/file");
    ///
    /// // Create a workspace.
    /// let mut workspace = Workspace::new(directory_path).unwrap();
    ///
    /// // Add a buffer to the workspace.
    /// let buf = Buffer::from_file(file_path).unwrap();
    /// workspace.add_buffer(buf);
    ///
    /// // Select the previous buffer.
    /// workspace.previous_buffer();
    /// ```
    pub fn previous_buffer(&mut self) {
        match self.current_buffer_index {
            Some(index) => {
                if index > 0 {
                    self.current_buffer_index = Some(index-1);
                } else {
                    self.current_buffer_index = Some(self.buffers.len()-1);
                }
            },
            None => return,
        }
    }

    /// Selects the next buffer in the workspace (buffers are ordered as
    /// they are added to the workspace). If the currently selected buffer is
    /// the last in the collection, this will wrap and select the first buffer.
    ///
    /// # Examples
    ///
    /// ```
    /// use scribe::Buffer;
    /// use scribe::Workspace;
    /// use std::path::Path;
    ///
    /// // Set up the paths we'll use.
    /// let directory_path = Path::new("tests/sample");
    /// let file_path = Path::new("tests/sample/file");
    ///
    /// // Create a workspace.
    /// let mut workspace = Workspace::new(directory_path).unwrap();
    ///
    /// // Add a buffer to the workspace.
    /// let buf = Buffer::from_file(file_path).unwrap();
    /// workspace.add_buffer(buf);
    ///
    /// // Select the next buffer.
    /// workspace.next_buffer();
    /// ```
    pub fn next_buffer(&mut self) {
        match self.current_buffer_index {
            Some(index) => {
                if index == self.buffers.len()-1 {
                    self.current_buffer_index = Some(0);
                } else {
                    self.current_buffer_index = Some(index+1);
                }
            },
            None => return,
        }
    }

    /// Whether or not the workspace contains a buffer with the specified path.
    ///
    /// # Examples
    ///
    /// ```
    /// use scribe::Buffer;
    /// use scribe::Workspace;
    /// use std::path::Path;
    ///
    /// // Set up the paths we'll use.
    /// let directory_path = Path::new("tests/sample");
    /// let file_path = Path::new("tests/sample/file");
    ///
    /// // Create a workspace.
    /// let mut workspace = Workspace::new(directory_path).unwrap();
    ///
    /// // Add a buffer to the workspace.
    /// let buf = Buffer::from_file(file_path.clone()).unwrap();
    /// workspace.add_buffer(buf);
    ///
    /// assert!(workspace.contains_buffer_with_path(&file_path));
    /// ```
    pub fn contains_buffer_with_path(&self, path: &Path) -> bool {
        if let Ok(ref canonical_path) = path.canonicalize() {
            self.buffers.iter().any(|buffer| {
                match buffer.path {
                    Some(ref buffer_path) => buffer_path == canonical_path,
                    None => false,
                }
            })
        } else {
            false
        }
    }

    // Returns a syntax definition based on the buffer's file extension,
    // falling back to a plain text definition if one cannot be found.
    fn find_syntax_definition(&self, buffer: &Buffer) -> Option<SyntaxDefinition> {
        // Find the syntax definition using the buffer's file extension.
        buffer.path.as_ref().and_then(|path|
            path.to_str().and_then(|p| p.split('.').last()).and_then(|ex|
                self.syntax_set.find_syntax_by_extension(ex).and_then(|s|
                    Some(s.clone())
                )
            )
        ).or(
            // Fall back to a plain text definition.
            Some(self.syntax_set.find_syntax_plain_text().clone())
        )
    }
}

#[cfg(test)]
mod tests {
    use super::Workspace;
    use buffer::Buffer;
    use std::path::Path;

    #[test]
    fn add_buffer_adds_and_selects_the_passed_buffer() {
        let mut workspace = Workspace::new(Path::new("tests/sample")).unwrap();
        let buf = Buffer::from_file(Path::new("tests/sample/file")).unwrap();
        workspace.add_buffer(buf);

        assert_eq!(workspace.buffers.len(), 1);
        assert_eq!(workspace.current_buffer().unwrap().data(), "it works!\n");
    }

    #[test]
    fn add_buffer_inserts_the_new_buffer_after_the_current_buffer() {
        let mut workspace = Workspace::new(Path::new("tests/sample")).unwrap();
        let mut buf1 = Buffer::new();
        let mut buf2 = Buffer::new();
        let mut buf3 = Buffer::new();
        buf1.insert("one");
        buf2.insert("two");
        buf3.insert("three");
        workspace.add_buffer(buf1);
        workspace.add_buffer(buf2);

        // Move to the first buffer.
        workspace.previous_buffer();

        // Insert the last buffer.
        workspace.add_buffer(buf3);

        // Make sure the newly inserted buffer was inserted after the current buffer.
        workspace.previous_buffer();
        assert_eq!(workspace.current_buffer().unwrap().data(), "one");
    }

    #[test]
    fn add_buffer_populates_buffers_with_unique_id_values() {
        let mut workspace = Workspace::new(Path::new("tests/sample")).unwrap();
        let buf1 = Buffer::new();
        let buf2 = Buffer::new();
        let buf3 = Buffer::new();

        workspace.add_buffer(buf1);
        assert_eq!(workspace.current_buffer().unwrap().id.unwrap(), 0);

        workspace.add_buffer(buf2);
        assert_eq!(workspace.current_buffer().unwrap().id.unwrap(), 1);

        workspace.add_buffer(buf3);
        assert_eq!(workspace.current_buffer().unwrap().id.unwrap(), 2);
    }

    #[test]
    fn add_buffer_populates_buffers_without_paths_using_plain_text_syntax() {
        let mut workspace = Workspace::new(Path::new("tests/sample")).unwrap();
        let buf = Buffer::new();
        workspace.add_buffer(buf);

        let name = workspace
          .current_buffer()
          .and_then(|ref b| b.syntax_definition.as_ref().map(|sd| sd.name.clone()));

        assert!(workspace.current_buffer().unwrap().syntax_definition.is_some());
        assert_eq!(name, Some("Plain Text".to_string()));
    }

    #[test]
    fn add_buffer_populates_buffers_with_unknown_extensions_using_plain_text_syntax() {
        let mut workspace = Workspace::new(Path::new("tests/sample")).unwrap();
        let buf = Buffer::from_file(Path::new("tests/sample/file"));
        workspace.add_buffer(buf.unwrap());

        let name = workspace
          .current_buffer()
          .and_then(|ref b| b.syntax_definition.as_ref().map(|sd| sd.name.clone()));

        assert!(workspace.current_buffer().unwrap().syntax_definition.is_some());
        assert_eq!(name, Some("Plain Text".to_string()));
    }

    #[test]
    fn open_buffer_adds_and_selects_the_buffer_at_the_specified_path() {
        let mut workspace = Workspace::new(Path::new("tests/sample")).unwrap();
        workspace.open_buffer(Path::new("tests/sample/file"));

        assert_eq!(workspace.buffers.len(), 1);
        assert_eq!(workspace.current_buffer().unwrap().data(), "it works!\n");
    }

    #[test]
    fn open_buffer_does_not_open_a_buffer_already_in_the_workspace() {
        let mut workspace = Workspace::new(Path::new("tests/sample")).unwrap();
        let thing = Path::new("tests/sample/file");
        workspace.open_buffer(Path::new("tests/sample/file"));
        workspace.open_buffer(Path::new("tests/sample/file"));

        assert_eq!(workspace.buffers.len(), 1);
    }

    #[test]
    fn open_buffer_selects_buffer_if_it_already_exists_in_workspace() {
        let mut workspace = Workspace::new(Path::new("tests/sample")).unwrap();
        workspace.open_buffer(Path::new("tests/sample/file"));

        // Add and select another buffer.
        let mut buf = Buffer::new();
        buf.insert("scribe");
        workspace.add_buffer(buf);
        assert_eq!(workspace.current_buffer().unwrap().data(), "scribe");

        // Try to add the first buffer again.
        workspace.open_buffer(Path::new("tests/sample/file"));

        // Ensure there are only two buffers, and that the
        // one requested via open_buffer is now selected.
        assert_eq!(workspace.buffers.len(), 2);
        assert_eq!(workspace.current_buffer().unwrap().data(), "it works!\n");
    }

    #[test]
    fn current_buffer_returns_none_when_there_are_no_buffers() {
        let mut workspace = Workspace::new(Path::new("tests/sample")).unwrap();
        assert!(workspace.current_buffer().is_none());
    }

    #[test]
    fn current_buffer_returns_one_when_there_are_buffers() {
        let mut workspace = Workspace::new(Path::new("tests/sample")).unwrap();
        let buf = Buffer::from_file(Path::new("tests/sample/file")).unwrap();
        workspace.add_buffer(buf);
        assert!(workspace.current_buffer().is_some());
    }

    #[test]
    fn close_current_buffer_does_nothing_when_none_are_open() {
        let mut workspace = Workspace::new(Path::new("tests/sample")).unwrap();
        workspace.close_current_buffer();
        assert!(workspace.current_buffer().is_none());
    }

    #[test]
    fn close_current_buffer_cleans_up_when_only_one_buffer_is_open() {
        let mut workspace = Workspace::new(Path::new("tests/sample")).unwrap();
        workspace.add_buffer(Buffer::new());
        workspace.close_current_buffer();
        assert!(workspace.current_buffer().is_none());
        assert!(workspace.current_buffer_index.is_none());
    }

    #[test]
    fn close_current_buffer_selects_the_previous_buffer() {
        let mut workspace = Workspace::new(Path::new("tests/sample")).unwrap();

        // Create two buffers and add them to the workspace.
        let mut first_buffer = Buffer::new();
        let mut second_buffer = Buffer::new();
        let mut third_buffer = Buffer::new();
        first_buffer.insert("first buffer");
        second_buffer.insert("second buffer");
        third_buffer.insert("second buffer");
        workspace.add_buffer(first_buffer);
        workspace.add_buffer(second_buffer);
        workspace.add_buffer(third_buffer);

        // Select the second buffer to make sure we
        // don't simply select the last buffer in the set.
        workspace.previous_buffer();

        workspace.close_current_buffer();
        assert_eq!(workspace.current_buffer().unwrap().data(), "first buffer");
    }

    #[test]
    fn close_current_buffer_selects_the_next_buffer_when_current_is_at_start() {
        let mut workspace = Workspace::new(Path::new("tests/sample")).unwrap();

        // Create two buffers and add them to the workspace.
        let mut first_buffer = Buffer::new();
        let mut second_buffer = Buffer::new();
        let mut third_buffer = Buffer::new();
        first_buffer.insert("first buffer");
        second_buffer.insert("second buffer");
        third_buffer.insert("second buffer");
        workspace.add_buffer(first_buffer);
        workspace.add_buffer(second_buffer);
        workspace.add_buffer(third_buffer);

        // Select the first buffer.
        workspace.previous_buffer();
        workspace.previous_buffer();

        workspace.close_current_buffer();
        assert_eq!(workspace.current_buffer().unwrap().data(), "second buffer");
    }

    #[test]
    fn previous_buffer_does_nothing_when_no_buffers_are_open() {
        let mut workspace = Workspace::new(Path::new("tests/sample")).unwrap();
        workspace.previous_buffer();
        assert!(workspace.current_buffer().is_none());
    }

    #[test]
    fn previous_buffer_when_three_are_open_selects_previous_wrapping_to_last() {
        let mut workspace = Workspace::new(Path::new("tests/sample")).unwrap();

        // Create two buffers and add them to the workspace.
        let mut first_buffer = Buffer::new();
        let mut second_buffer = Buffer::new();
        let mut third_buffer = Buffer::new();
        first_buffer.insert("first buffer");
        second_buffer.insert("second buffer");
        third_buffer.insert("third buffer");
        workspace.add_buffer(first_buffer);
        workspace.add_buffer(second_buffer);
        workspace.add_buffer(third_buffer);

        // Ensure that the third buffer is currently selected.
        assert_eq!(workspace.current_buffer().unwrap().data(), "third buffer");

        // Ensure that the second buffer is returned.
        workspace.previous_buffer();
        assert_eq!(workspace.current_buffer().unwrap().data(), "second buffer");

        // Ensure that the first buffer is returned.
        workspace.previous_buffer();
        assert_eq!(workspace.current_buffer().unwrap().data(), "first buffer");

        // Ensure that it wraps back to the third buffer.
        workspace.previous_buffer();
        assert_eq!(workspace.current_buffer().unwrap().data(), "third buffer");
    }

    #[test]
    fn next_buffer_does_nothing_when_no_buffers_are_open() {
        let mut workspace = Workspace::new(Path::new("tests/sample")).unwrap();
        workspace.next_buffer();
        assert!(workspace.current_buffer().is_none());
    }

    #[test]
    fn next_buffer_when_three_are_open_selects_next_wrapping_to_first() {
        let mut workspace = Workspace::new(Path::new("tests/sample")).unwrap();

        // Create two buffers and add them to the workspace.
        let mut first_buffer = Buffer::new();
        let mut second_buffer = Buffer::new();
        let mut third_buffer = Buffer::new();
        first_buffer.insert("first buffer");
        second_buffer.insert("second buffer");
        third_buffer.insert("third buffer");
        workspace.add_buffer(first_buffer);
        workspace.add_buffer(second_buffer);
        workspace.add_buffer(third_buffer);

        // Ensure that the third buffer is currently selected.
        assert_eq!(workspace.current_buffer().unwrap().data(), "third buffer");

        // Ensure that it wraps back to the first buffer.
        workspace.next_buffer();
        assert_eq!(workspace.current_buffer().unwrap().data(), "first buffer");

        // Ensure that the second buffer is returned.
        workspace.next_buffer();
        assert_eq!(workspace.current_buffer().unwrap().data(), "second buffer");

        // Ensure that the third buffer is returned.
        workspace.next_buffer();
        assert_eq!(workspace.current_buffer().unwrap().data(), "third buffer");
    }
}
