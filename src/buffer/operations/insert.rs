use buffer::operation::Operation;
use buffer::gap_buffer::GapBuffer;
use buffer::{Position, Range};
use std::clone::Clone;

#[derive(Clone)]
pub struct Insert {
    content: String,
    position: Position,
}

impl Operation for Insert {
    fn run(&mut self, buffer: &mut GapBuffer) {
        buffer.insert(&self.content, &self.position);
    }

    // We need to calculate the range of the inserted content.
    // The start of the range corresponds to the cursor position at the time of the insert,
    // which we've stored. Finding the end of the range requires that we dig into the content.
    fn reverse(&mut self, buffer: &mut GapBuffer) {
        // The line count of the content tells us the line number for the end of the
        // range (just add the number of new lines to the starting line).
        let line_count = self.content.chars().filter(|&c| c == '\n').count() + 1;
        let end_line = self.position.line + line_count - 1;

        let end_offset = if line_count == 1 {
            // If there's only one line, the range starts and ends on the same line, and so its
            // offset needs to take the original insertion location into consideration.
            self.position.offset + self.content.len()
        } else {
            // If there are multiple lines, the end of the range doesn't
            // need to consider the original insertion location.
            match self.content.lines().last() {
                Some(line) => line.len(),
                None => return,
            }
        };

        // Now that we have the required info,
        // build the end position and total range.
        let end_position = Position{
            line: end_line,
            offset: end_offset,
        };
        let range = Range{
            start: self.position.clone(),
            end: end_position,
        };

        // Remove the content we'd previously inserted.
        buffer.delete(&range);
    }

    fn clone_operation(&self) -> Box<Operation> {
        Box::new(self.clone())
    }
}

pub fn new(content: String, position: Position) -> Insert {
    Insert{ content: content, position: position }
}

#[cfg(test)]
mod tests {
    use super::new;
    use buffer::position::Position;
    use buffer::operation::Operation;

    #[test]
    fn run_and_reverse_add_and_remove_content_without_newlines_at_cursor_position() {
        // Set up a buffer with some data.
        let mut buffer = ::buffer::gap_buffer::new(String::new());
        let start_position = Position{ line: 0, offset: 0 };
        buffer.insert(&"something", &start_position);

        // Set up a position pointing to the end of the buffer's contents.
        let insert_position = Position{ line: 0, offset: 9 };

        // Create the insert operation and run it.
        let mut insert_operation = super::new(" else".to_string(), insert_position);
        insert_operation.run(&mut buffer);

        assert_eq!(buffer.to_string(), "something else");

        insert_operation.reverse(&mut buffer);

        assert_eq!(buffer.to_string(), "something");
    }

    #[test]
    fn run_and_reverse_add_and_remove_content_with_newlines_at_cursor_position() {
        // Set up a buffer with some data.
        let mut buffer = ::buffer::gap_buffer::new(String::new());
        let start_position = Position{ line: 0, offset: 0 };
        buffer.insert(&"\n something", &start_position);

        // Set up a position pointing to the end of the buffer's contents.
        let insert_position = Position{ line: 1, offset: 10 };

        // Create the insert operation and run it.
        //
        // NOTE: The newline character ensures that the operation doesn't use a naive
        //       algorithm based purely on the content length.
        let mut insert_operation = super::new("\n else\n entirely".to_string(), insert_position);
        insert_operation.run(&mut buffer);

        assert_eq!(buffer.to_string(), "\n something\n else\n entirely");

        insert_operation.reverse(&mut buffer);

        assert_eq!(buffer.to_string(), "\n something");
    }

    #[test]
    fn reverse_removes_a_newline() {
        // Set up a buffer with some data.
        let mut buffer = ::buffer::gap_buffer::new(String::new());
        let mut insert_operation = super::new("\n".to_string(), Position{ line: 0, offset: 0 });
        insert_operation.run(&mut buffer);
        assert_eq!(buffer.to_string(), "\n");

        insert_operation.reverse(&mut buffer);
        assert_eq!(buffer.to_string(), "");
    }
}
