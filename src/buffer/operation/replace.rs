use crate::buffer::operation::Operation;
use crate::buffer::{Buffer, Cursor, GapBuffer, Position};
use std::cell::RefCell;
use std::clone::Clone;
use std::convert::Into;
use std::rc::Rc;

/// A reversible buffer insert operation.
///
/// Inserts the provided content at the specified position. Tracks both, and reverses
/// the operation by calculating the content's start and end positions (range), relative
/// to its inserted location, and removing said range from the underlying buffer.
///
/// If the buffer is configured with a `change_callback`, it will be called with
/// the position of this operation when it is run or reversed.
#[derive(Clone)]
pub struct Replace {
    old_content: String,
    new_content: String,
}

impl Operation for Replace {
    fn run(&mut self, buffer: &mut Buffer) {
        replace_content(self.new_content.clone(), buffer);
    }

    fn reverse(&mut self, buffer: &mut Buffer) {
        replace_content(self.old_content.clone(), buffer);
    }

    fn clone_operation(&self) -> Box<dyn Operation> {
        Box::new(self.clone())
    }
}

impl Replace {
    /// Creates a new empty insert operation.
    pub fn new(old_content: String, new_content: String) -> Replace {
        Replace {
            old_content,
            new_content,
        }
    }
}

impl Buffer {
    /// Replaces the buffer's contents with the provided data. This method will
    /// make best efforts to retain the full cursor position, then cursor line,
    /// and will ultimately fall back to resetting the cursor to its initial
    /// (0,0) position if these fail. The buffer's ID, syntax definition, and
    /// change callback are always persisted.
    ///
    /// <div class="warning">
    ///   As this is a reversible operation, both the before and after buffer
    ///   contents are kept in-memory, which for large buffers may be relatively
    ///   expensive. To help avoid needless replacements, this method will
    ///   ignore requests that don't actually change content. Despite this, use
    ///   this operation judiciously; it is designed for wholesale replacements
    ///   (e.g. external formatting tools) that cannot be broken down into
    ///   selective delete/insert operations.
    /// </div>
    ///
    /// # Examples
    ///
    /// ```
    /// use scribe::buffer::{Buffer, Position};
    ///
    /// let mut buffer = Buffer::new();
    /// buffer.insert("scribe\nlibrary\n");
    /// buffer.cursor.move_to(Position { line: 1, offset: 1 });
    /// buffer.replace("new\ncontent");
    ///
    /// assert_eq!(buffer.data(), "new\ncontent");
    /// assert_eq!(*buffer.cursor, Position{ line: 1, offset: 1 });
    /// ```
    pub fn replace<T: Into<String> + AsRef<str>>(&mut self, content: T) {
        let old_content = self.data();

        // Ignore replacements that don't change content.
        if content.as_ref() == old_content {
            return;
        }

        // Build and run an insert operation.
        let mut op = Replace::new(self.data(), content.into());
        op.run(self);

        // Store the operation in the history object so that it can be undone.
        match self.operation_group {
            Some(ref mut group) => group.add(Box::new(op)),
            None => self.history.add(Box::new(op)),
        };
    }
}

fn replace_content(content: String, buffer: &mut Buffer) {
    // Create a new gap buffer and associated cursor with the new content.
    let data = Rc::new(RefCell::new(GapBuffer::new(content)));
    let mut cursor = Cursor::new(data.clone(), Position { line: 0, offset: 0 });

    // Try to retain cursor position or line of the current gap buffer.
    if !cursor.move_to(*buffer.cursor) {
        cursor.move_to(Position {
            line: buffer.cursor.line,
            offset: 0,
        });
    }

    // Do the replacement.
    buffer.data = data;
    buffer.cursor = cursor;

    // Run the change callback, if present.
    if let Some(ref callback) = buffer.change_callback {
        callback(Position::new())
    }
}

#[cfg(test)]
mod tests {
    use crate::buffer::position::Position;
    use crate::buffer::Buffer;
    use std::cell::RefCell;
    use std::path::Path;
    use std::rc::Rc;

    #[test]
    fn replace_retains_full_position_when_possible() {
        let mut buffer = Buffer::new();
        buffer.insert("amp editor");

        // Move to a position that will exist after replacing content.
        buffer.cursor.move_to(Position { line: 0, offset: 3 });

        // Replace the buffer content.
        buffer.replace("scribe");

        // Verify that the position is retained.
        assert_eq!(*buffer.cursor, Position { line: 0, offset: 3 });
    }

    #[test]
    fn replace_retains_position_line_when_possible() {
        let mut buffer = Buffer::new();

        // Move to a position whose line (but not offset)
        // is available in the replaced content.
        buffer.insert("amp\neditor");
        buffer.cursor.move_to(Position { line: 1, offset: 1 });

        // Replace the buffer content.
        buffer.replace("scribe\n");

        // Verify that the position is set to the start of the same line.
        assert_eq!(*buffer.cursor, Position { line: 1, offset: 0 });
    }

    #[test]
    fn replace_discards_position_when_impossible() {
        let mut buffer = Buffer::new();

        // Move to a position entirely unavailable in the replaced content.
        buffer.insert("\namp\neditor");
        buffer.cursor.move_to(Position { line: 2, offset: 1 });

        // Replace the buffer content.
        buffer.replace("scribe\n");

        // Verify that the position is discarded.
        assert_eq!(*buffer.cursor, Position::new());
    }

    #[test]
    fn replace_calls_change_callback_with_zero_position() {
        let mut buffer = Buffer::new();
        buffer.insert("amp\neditor");

        // Create a non-zero position that we'll share with the callback.
        let tracked_position = Rc::new(RefCell::new(Position { line: 1, offset: 1 }));
        let callback_position = tracked_position.clone();

        // Set up the callback so that it updates the shared position.
        buffer.change_callback = Some(Box::new(move |change_position| {
            *callback_position.borrow_mut() = change_position
        }));

        // Replace the buffer content.
        buffer.replace("scribe");

        // Verify that the callback received the correct position.
        assert_eq!(*tracked_position.borrow(), Position::new());
    }

    #[test]
    fn replace_flags_buffer_as_modified() {
        let file_path = Path::new("tests/sample/file");
        let mut buffer = Buffer::from_file(file_path).unwrap();

        // Replace the buffer content.
        buffer.replace("scribe");

        // Verify that the buffer is seen as modified.
        assert!(buffer.modified());
    }

    #[test]
    fn replace_is_reversible() {
        let file_path = Path::new("tests/sample/file");
        let mut buffer = Buffer::from_file(file_path).unwrap();

        // Replace the buffer content and then undo it.
        buffer.replace("scribe");
        buffer.undo();

        // Verify that the original content is restored.
        assert_eq!(buffer.data(), "it works!\n");
    }

    #[test]
    fn replace_does_nothing_if_replacement_matches_buffer_contents() {
        let file_path = Path::new("tests/sample/file");
        let mut buffer = Buffer::from_file(file_path).unwrap();

        // Try to replace buffer content with matching content.
        buffer.replace("it works!\n");

        assert!(!buffer.modified());
        assert!(buffer.history.previous().is_none());
    }
}
