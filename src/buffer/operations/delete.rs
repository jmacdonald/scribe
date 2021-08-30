use buffer::operation::Operation;
use buffer::{Buffer, Position, Range};
use std::clone::Clone;

/// A reversible buffer delete operation.
///
/// Deletes the content at the specified range. Tracks the deleted content and specified
/// range, and reverses the operation by (trivially) inserting the deleted content at
/// the start of the specified range.
///
/// If the buffer is configured with a `change_callback`, it will be called with
/// the starting position of this operation when it is run or reversed.
#[derive(Clone)]
pub struct Delete {
    content: Option<String>,
    range: Range,
}

impl Operation for Delete {
    fn run(&mut self, buffer: &mut Buffer) {
        // Fetch and store the content we're about to delete.
        self.content = buffer.data.borrow().read(&self.range);

        // Delete the data.
        buffer.data.borrow_mut().delete(&self.range);

        // Run the change callback, if present.
        if let Some(ref callback) = buffer.change_callback {
            callback(self.range.start())
        }
    }

    fn reverse(&mut self, buffer: &mut Buffer) {
        if let Some(ref content) = self.content {
            buffer.data.borrow_mut().insert(content, &self.range.start());

            // Run the change callback, if present.
            if let Some(ref callback) = buffer.change_callback {
                callback(self.range.start())
            }
        }
    }

    fn clone_operation(&self) -> Box<dyn Operation> {
        Box::new(self.clone())
    }
}

impl Delete {
    /// Creates a new empty delete operation.
    pub fn new(range: Range) -> Delete {
        Delete{ content: None, range }
    }
}

impl Buffer {
    /// Deletes a character at the cursor position. If at the end
    /// of the current line, it'll try to delete a newline character
    /// (joining the lines), succeeding if there's a line below.
    ///
    /// # Examples
    ///
    /// ```
    /// use scribe::Buffer;
    ///
    /// let mut buffer = Buffer::new();
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

        // The range we're building is going to be consumed,
        // so create a clone of the cursor's current position.
        let start = self.cursor.position;

        // Now that we've established the range, defer.
        self.delete_range(Range::new(start, end));
    }

    /// Removes a range of characters from the buffer.
    ///
    /// # Examples
    ///
    /// ```
    /// use scribe::Buffer;
    /// use scribe::buffer::{Position, Range};
    ///
    /// // Set up an example buffer.
    /// let mut buffer = Buffer::new();
    /// buffer.insert("scribe library");
    ///
    /// // Set up the range we'd like to delete.
    /// let start = Position{ line: 0, offset: 6 };
    /// let end = Position{ line: 0, offset: 14 };
    /// let range = Range::new(start, end);
    ///
    /// buffer.delete_range(range);
    ///
    /// assert_eq!(buffer.data(), "scribe");
    /// ```
    pub fn delete_range(&mut self, range: Range) {
        // Build and run a delete operation.
        let mut op = Delete::new(range);
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
    use super::Delete;
    use buffer::{Buffer, Position, Range};
    use buffer::operation::Operation;

    #[test]
    fn run_and_reverse_remove_and_add_content_without_newlines_at_cursor_position() {
        // Set up a buffer with some data.
        let mut buffer = Buffer::new();
        buffer.insert("something else");

        // Set up a range that covers everything after the first word.
        let start = Position{ line: 0, offset: 9 };
        let end = Position{ line: 0, offset: 14 };
        let delete_range = Range::new(start, end);

        // Create the delete operation and run it.
        let mut delete_operation = Delete::new(delete_range);
        delete_operation.run(&mut buffer);

        assert_eq!(buffer.data(), "something");

        delete_operation.reverse(&mut buffer);

        assert_eq!(buffer.data(), "something else");
    }

    #[test]
    fn run_and_reverse_remove_and_add_content_with_newlines_at_cursor_position() {
        // Set up a buffer with some data.
        let mut buffer = Buffer::new();
        buffer.insert("\n something\n else\n entirely");

        // Set up a range that covers everything after the first word.
        let start = Position{ line: 1, offset: 10 };
        let end = Position{ line: 3, offset: 9 };
        let delete_range = Range::new(start, end);

        // Create the delete operation and run it.
        //
        // NOTE: The newline character ensures that the operation doesn't use a naive
        //       algorithm based purely on the content length.
        let mut delete_operation = Delete::new(delete_range);
        delete_operation.run(&mut buffer);

        assert_eq!(buffer.data(), "\n something");

        delete_operation.reverse(&mut buffer);

        assert_eq!(buffer.data(), "\n something\n else\n entirely");
    }

    #[test]
    fn run_calls_change_callback_with_position() {
        // Set up a buffer with some data.
        let mut buffer = Buffer::new();
        buffer.insert("something else");

        // Set up a range that covers everything after the first word.
        let start = Position{ line: 0, offset: 9 };
        let end = Position{ line: 0, offset: 14 };
        let delete_range = Range::new(start, end);

        // Create a position that we'll share with the callback.
        let tracked_position = Rc::new(RefCell::new(Position::new()));
        let callback_position = tracked_position.clone();

        // Set up the callback so that it updates the shared position.
        buffer.change_callback = Some(Box::new(move |change_position| {
            *callback_position.borrow_mut() = change_position
        }));

        // Create the delete operation and run it.
        let mut delete_operation = Delete::new(delete_range);
        delete_operation.run(&mut buffer);

        // Verify that the callback received the correct position.
        assert_eq!(*tracked_position.borrow(), Position{ line: 0, offset: 9});
    }

    #[test]
    fn reverse_calls_change_callback_with_position() {
        // Set up a buffer with some data.
        let mut buffer = Buffer::new();
        buffer.insert("something else");

        // Set up a range that covers everything after the first word.
        let start = Position{ line: 0, offset: 9 };
        let end = Position{ line: 0, offset: 14 };
        let delete_range = Range::new(start, end);

        // Create the delete operation and run it.
        let mut delete_operation = Delete::new(delete_range);
        delete_operation.run(&mut buffer);

        // Create a position that we'll share with the callback.
        let tracked_position = Rc::new(RefCell::new(Position::new()));
        let callback_position = tracked_position.clone();

        // Set up the callback so that it updates the shared position.
        buffer.change_callback = Some(Box::new(move |change_position| {
            *callback_position.borrow_mut() = change_position
        }));

        // Reverse the operation.
        delete_operation.reverse(&mut buffer);

        // Verify that the callback received the correct position.
        assert_eq!(*tracked_position.borrow(), Position{ line: 0, offset: 9});
    }
}
