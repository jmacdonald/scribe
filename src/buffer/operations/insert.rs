use buffer::operation::Operation;
use buffer::{Buffer, Position, Range};
use std::clone::Clone;

/// A reversible buffer insert operation.
///
/// Inserts the provided content at the specified position. Tracks both, and reverses
/// the operation by calculating the content's start and end positions (range), relative
/// to its inserted location, and removing said range from the underlying buffer.
#[derive(Clone)]
pub struct Insert {
    content: String,
    position: Position,
}

impl Operation for Insert {
    fn run(&mut self, buffer: &mut Buffer) {
        buffer.data.borrow_mut().insert(&self.content, &self.position);

        // We've modified the buffer, but it doesn't know that. Bust its cache.
        buffer.clear_caches()
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
            self.position.offset + self.content.chars().count()
        } else {
            // If there are multiple lines, the end of the range doesn't
            // need to consider the original insertion location.
            match self.content.split("\n").last() {
                Some(line) => line.chars().count(),
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
            self.position.clone(),
            end_position
        );

        // Remove the content we'd previously inserted.
        buffer.data.borrow_mut().delete(&range);

        // We've modified the buffer, but it doesn't know that. Bust its cache.
        buffer.clear_caches()
    }

    fn clone_operation(&self) -> Box<Operation> {
        Box::new(self.clone())
    }
}

impl Insert {
    /// Creates a new empty insert operation.
    pub fn new(content: String, position: Position) -> Insert {
        Insert{ content: content, position: position }
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
    pub fn insert(&mut self, data: &str) {
        // Build and run an insert operation.
        let mut op = Insert::new(data.to_string(), self.cursor.position.clone());
        op.run(self);

        // Store the operation in the history
        // object so that it can be undone.
        match self.operation_group {
            Some(ref mut group) => group.add(Box::new(op)),
            None => self.history.add(Box::new(op)),
        };

        // Caches are invalid as the buffer has changed.
        self.clear_caches();
    }
}

#[cfg(test)]
mod tests {
    use super::Insert;
    use buffer::Buffer;
    use buffer::position::Position;
    use buffer::operation::Operation;

    #[test]
    fn run_and_reverse_add_and_remove_content_without_newlines_at_cursor_position() {
        // Set up a buffer with some data.
        let mut buffer = Buffer::new();
        buffer.insert(&"something");

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
        buffer.insert(&"\n something");

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
        buffer.insert(&"scribe\nlibrary\n");

        let mut insert_operation = Insert::new("editor\n".to_string(), Position{ line: 1, offset: 0 });
        insert_operation.run(&mut buffer);
        assert_eq!(buffer.data(), "scribe\neditor\nlibrary\n");

        insert_operation.reverse(&mut buffer);
        assert_eq!(buffer.data(), "scribe\nlibrary\n");
    }
}
