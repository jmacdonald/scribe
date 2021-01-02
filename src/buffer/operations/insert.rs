use buffer::operation::Operation;
use buffer::{Buffer, Position, Range};
use std::clone::Clone;
use std::convert::Into;
use unicode_segmentation::UnicodeSegmentation;

/// A reversible buffer insert operation.
///
/// Inserts the provided content at the specified position. Tracks both, and reverses
/// the operation by calculating the content's start and end positions (range), relative
/// to its inserted location, and removing said range from the underlying buffer.
///
/// If the buffer is configured with a `change_callback`, it will be called with
/// the position of this operation when it is run or reversed.
#[derive(Clone)]
pub struct Insert {
    content: String,
    position: Position,
}

impl Operation for Insert {
    fn run(&mut self, buffer: &mut Buffer) {
        buffer.data.borrow_mut().insert(&self.content, &self.position);

        // Run the change callback, if present.
        if let Some(ref callback) = buffer.change_callback {
            callback(self.position)
        }
    }

    // We need to calculate the range of the inserted content.
    // The start of the range corresponds to the cursor position at the time of the insert,
    // which we've stored. Finding the end of the range requires that we dig into the content.
    fn reverse(&mut self, buffer: &mut Buffer) {
        // The line count of the content tells us the line number for the end of the
        // range (just add the number of new lines to the starting line).
        let line_count = self.content.chars().filter(|&c| c == '\n').count() + 1;
        let end_line = self.position.line + line_count - 1;

        let end_offset = if line_count == 1 {
            // If there's only one line, the range starts and ends on the same line, and so its
            // offset needs to take the original insertion location into consideration.
            self.position.offset + self.content.graphemes(true).count()
        } else {
            // If there are multiple lines, the end of the range doesn't
            // need to consider the original insertion location.
            match self.content.split('\n').last() {
                Some(line) => line.graphemes(true).count(),
                None => return,
            }
        };

        // Now that we have the required info,
        // build the end position and total range.
        let end_position = Position{
            line: end_line,
            offset: end_offset,
        };
        let range = Range::new(
            self.position,
            end_position
        );

        // Remove the content we'd previously inserted.
        buffer.data.borrow_mut().delete(&range);

        // Run the change callback, if present.
        if let Some(ref callback) = buffer.change_callback {
            callback(self.position)
        }
    }

    fn clone_operation(&self) -> Box<dyn Operation> {
        Box::new(self.clone())
    }
}

impl Insert {
    /// Creates a new empty insert operation.
    pub fn new(content: String, position: Position) -> Insert {
        Insert{ content, position }
    }
}

impl Buffer {
    /// Inserts `data` into the buffer at the cursor position.
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
    pub fn insert<T: Into<String>>(&mut self, data: T) {
        // Build and run an insert operation.
        let mut op = Insert::new(data.into(), self.cursor.position);
        op.run(self);

        // Store the operation in the history
        // object so that it can be undone.
        match self.operation_group {
            Some(ref mut group) => group.add(Box::new(op)),
            None => self.history.add(Box::new(op)),
        };
    }
}

#[cfg(test)]
mod tests {
    use std::cell::RefCell;
    use std::rc::Rc;
    use super::Insert;
    use buffer::Buffer;
    use buffer::position::Position;
    use buffer::operation::Operation;

    #[test]
    fn run_and_reverse_add_and_remove_content_without_newlines_at_cursor_position() {
        // Set up a buffer with some data.
        let mut buffer = Buffer::new();
        buffer.insert("something");

        // Set up a position pointing to the end of the buffer's contents.
        let insert_position = Position{ line: 0, offset: 9 };

        // Create the insert operation and run it.
        let mut insert_operation = Insert::new(" else".to_string(), insert_position);
        insert_operation.run(&mut buffer);

        assert_eq!(buffer.data(), "something else");

        insert_operation.reverse(&mut buffer);

        assert_eq!(buffer.data(), "something");
    }

    #[test]
    fn run_and_reverse_add_and_remove_content_with_newlines_at_cursor_position() {
        // Set up a buffer with some data.
        let mut buffer = Buffer::new();
        buffer.insert("\n something");

        // Set up a position pointing to the end of the buffer's contents.
        let insert_position = Position{ line: 1, offset: 10 };

        // Create the insert operation and run it.
        //
        // NOTE: The newline character ensures that the operation doesn't use a naive
        //       algorithm based purely on the content length.
        let mut insert_operation = Insert::new("\n else\n entirely".to_string(), insert_position);
        insert_operation.run(&mut buffer);

        assert_eq!(buffer.data(), "\n something\n else\n entirely");

        insert_operation.reverse(&mut buffer);

        assert_eq!(buffer.data(), "\n something");
    }

    #[test]
    fn reverse_removes_a_newline() {
        // Set up a buffer with some data.
        let mut buffer = Buffer::new();
        let mut insert_operation = Insert::new("\n".to_string(), Position{ line: 0, offset: 0 });
        insert_operation.run(&mut buffer);
        assert_eq!(buffer.data(), "\n");

        insert_operation.reverse(&mut buffer);
        assert_eq!(buffer.data(), "");
    }

    #[test]
    fn reverse_correctly_removes_line_ranges() {
        // Set up a buffer with some data.
        let mut buffer = Buffer::new();
        buffer.insert("scribe\nlibrary\n");

        let mut insert_operation = Insert::new("editor\n".to_string(), Position{ line: 1, offset: 0 });
        insert_operation.run(&mut buffer);
        assert_eq!(buffer.data(), "scribe\neditor\nlibrary\n");

        insert_operation.reverse(&mut buffer);
        assert_eq!(buffer.data(), "scribe\nlibrary\n");
    }

    #[test]
    fn reverse_correctly_removes_single_line_content_with_graphemes() {
        // Set up a buffer with some data.
        let mut buffer = Buffer::new();
        buffer.insert("scribe\nlibrary");

        let mut insert_operation = Insert::new("नी editor ".to_string(), Position{ line: 1, offset: 0 });
        insert_operation.run(&mut buffer);
        assert_eq!(buffer.data(), "scribe\nनी editor library");

        insert_operation.reverse(&mut buffer);
        assert_eq!(buffer.data(), "scribe\nlibrary");
    }

    #[test]
    fn reverse_correctly_removes_multi_line_content_with_graphemes() {
        // Set up a buffer with some data.
        let mut buffer = Buffer::new();
        buffer.insert("scribe\nlibrary");

        let mut insert_operation = Insert::new("\nनी editor".to_string(), Position{ line: 0, offset: 6 });
        insert_operation.run(&mut buffer);
        assert_eq!(buffer.data(), "scribe\nनी editor\nlibrary");

        insert_operation.reverse(&mut buffer);
        assert_eq!(buffer.data(), "scribe\nlibrary");
    }

    #[test]
    fn run_calls_change_callback_with_position() {
        // Set up a buffer with some data.
        let mut buffer = Buffer::new();
        buffer.insert("something");

        // Set up a position pointing to the end of the buffer's contents.
        let insert_position = Position{ line: 0, offset: 9 };

        // Create a position that we'll share with the callback.
        let tracked_position = Rc::new(RefCell::new(Position::new()));
        let callback_position = tracked_position.clone();

        // Set up the callback so that it updates the shared position.
        buffer.change_callback = Some(Box::new(move |change_position| {
            *callback_position.borrow_mut() = change_position
        }));

        // Create the insert operation and run it.
        let mut insert_operation = Insert::new(" else".to_string(), insert_position);
        insert_operation.run(&mut buffer);

        // Verify that the callback received the correct position.
        assert_eq!(*tracked_position.borrow(), Position{ line: 0, offset: 9});
    }

    #[test]
    fn reverse_calls_change_callback_with_position() {
        // Set up a buffer with some data.
        let mut buffer = Buffer::new();
        buffer.insert("something");

        // Set up a position pointing to the end of the buffer's contents.
        let insert_position = Position{ line: 0, offset: 9 };

        // Create the insert operation and run it.
        let mut insert_operation = Insert::new(" else".to_string(), insert_position);
        insert_operation.run(&mut buffer);

        // Create a position that we'll share with the callback.
        let tracked_position = Rc::new(RefCell::new(Position::new()));
        let callback_position = tracked_position.clone();

        // Set up the callback so that it updates the shared position.
        buffer.change_callback = Some(Box::new(move |change_position| {
            *callback_position.borrow_mut() = change_position
        }));

        // Reverse the operation.
        insert_operation.reverse(&mut buffer);

        // Verify that the callback received the correct position.
        assert_eq!(*tracked_position.borrow(), Position{ line: 0, offset: 9});
    }
}
