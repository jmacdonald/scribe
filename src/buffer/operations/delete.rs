use buffer::operation::Operation;
use buffer::gap_buffer::GapBuffer;
use buffer::Range;
use std::clone::Clone;

#[derive(Clone)]
pub struct Delete {
    content: Option<String>,
    range: Range,
}

impl Operation for Delete {
    fn run(&mut self, buffer: &mut GapBuffer) {
        self.content = buffer.read(&self.range);
        buffer.delete(&self.range);
    }

    fn reverse(&mut self, buffer: &mut GapBuffer) {
        match self.content {
            Some(ref content) => buffer.insert(content, &self.range.start),
            None => (),
        }
    }

    fn clone_operation(&self) -> Box<Operation> {
        Box::new(self.clone())
    }
}

pub fn new(range: Range) -> Delete {
    Delete{ content: None, range: range }
}

#[cfg(test)]
mod tests {
    use super::new;
    use buffer::{Position, Range};
    use buffer::operation::Operation;

    #[test]
    fn run_and_reverse_remove_and_add_content_without_newlines_at_cursor_position() {
        // Set up a buffer with some data.
        let mut buffer = ::buffer::gap_buffer::new(String::new());
        let start_position = Position{ line: 0, offset: 0 };
        buffer.insert(&"something else", &start_position);

        // Set up a range that covers everything after the first word.
        let start = Position{ line: 0, offset: 9 };
        let end = Position{ line: 0, offset: 14 };
        let delete_range = Range{ start: start, end: end };

        // Create the delete operation and run it.
        let mut delete_operation = super::new(delete_range);
        delete_operation.run(&mut buffer);

        assert_eq!(buffer.to_string(), "something");

        delete_operation.reverse(&mut buffer);

        assert_eq!(buffer.to_string(), "something else");
    }

    #[test]
    fn run_and_reverse_remove_and_add_content_with_newlines_at_cursor_position() {
        // Set up a buffer with some data.
        let mut buffer = ::buffer::gap_buffer::new(String::new());
        let start_position = Position{ line: 0, offset: 0 };
        buffer.insert(&"\n something\n else\n entirely", &start_position);

        // Set up a range that covers everything after the first word.
        let start = Position{ line: 1, offset: 10 };
        let end = Position{ line: 3, offset: 9 };
        let delete_range = Range{ start: start, end: end };

        // Create the delete operation and run it.
        //
        // NOTE: The newline character ensures that the operation doesn't use a naive
        //       algorithm based purely on the content length.
        let mut delete_operation = super::new(delete_range);
        delete_operation.run(&mut buffer);

        assert_eq!(buffer.to_string(), "\n something");

        delete_operation.reverse(&mut buffer);

        assert_eq!(buffer.to_string(), "\n something\n else\n entirely");
    }
}
