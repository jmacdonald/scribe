use buffer::operation::Operation;
use buffer::{Buffer, Range};
use std::clone::Clone;

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

        // We've modified the buffer, but it doesn't know that. Bust its cache.
        buffer.clear_caches()
    }

    fn reverse(&mut self, buffer: &mut Buffer) {
        match self.content {
            Some(ref content) => {
                buffer.data.borrow_mut().insert(content, &self.range.start);

                // We've modified the buffer, but it doesn't know that. Bust its cache.
                buffer.clear_caches()
            },
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
        let mut buffer = ::buffer::new();
        buffer.insert(&"something else");

        // Set up a range that covers everything after the first word.
        let start = Position{ line: 0, offset: 9 };
        let end = Position{ line: 0, offset: 14 };
        let delete_range = Range{ start: start, end: end };

        // Create the delete operation and run it.
        let mut delete_operation = super::new(delete_range);
        delete_operation.run(&mut buffer);

        assert_eq!(buffer.data(), "something");

        delete_operation.reverse(&mut buffer);

        assert_eq!(buffer.data(), "something else");
    }

    #[test]
    fn run_and_reverse_remove_and_add_content_with_newlines_at_cursor_position() {
        // Set up a buffer with some data.
        let mut buffer = ::buffer::new();
        buffer.insert(&"\n something\n else\n entirely");

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

        assert_eq!(buffer.data(), "\n something");

        delete_operation.reverse(&mut buffer);

        assert_eq!(buffer.data(), "\n something\n else\n entirely");
    }
}
