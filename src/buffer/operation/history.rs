use buffer::operation::Operation;

/// Tracks a series of operations, which can be traversed backwards and forwards.
pub struct History {
    previous: Vec<Box<Operation>>,
    next: Vec<Box<Operation>>,
}

impl History {
    /// Store an operation that has already been run.
    pub fn add(&mut self, operation: Box<Operation>) {
        self.previous.push(operation);
    }

    /// Navigate the history backwards.
    pub fn previous(&mut self) -> Option<Box<Operation>> {
        match self.previous.pop() {
            Some(operation) => {
                // We've found a previous operation. Before we return it, store
                // a clone of it so that we can re-apply it as a redo operation.
                self.next.push(operation.clone_operation());
                Some(operation)
            },
            None => None
        }
    }

    /// Navigate the history forwards.
    pub fn next(&mut self) -> Option<Box<Operation>> {
        self.next.pop()
    }
}

pub fn new() -> History {
    History{ previous: Vec::new(), next: Vec::new() }
}

#[cfg(test)]
mod tests {
    use super::new;
    use buffer::{gap_buffer, Position, operations};
    use buffer::operation::Operation;

    #[test]
    fn previous_and_next_return_the_correct_operations() {
        let mut history = new();

        // Set up a buffer with some data.
        let mut buffer = gap_buffer::new(String::new());

        // Run an insert operation and add it to the history.
        let insert_position = Position{ line: 0, offset: 0 };
        let mut insert_operation = operations::insert::new("scribe".to_string(), insert_position);
        insert_operation.run(&mut buffer);
        history.add(Box::new(insert_operation));

        // Make sure the buffer has the inserted content.
        assert_eq!(buffer.to_string(), "scribe");

        // Pull and reverse the last history item.
        match history.previous() {
            Some(mut operation) => operation.reverse(&mut buffer),
            None => (),
        };

        // Make sure the buffer had the inserted content removed.
        assert_eq!(buffer.to_string(), "");

        // Pull and run the next history item.
        match history.next() {
            Some(mut operation) => operation.run(&mut buffer),
            None => (),
        };

        // Make sure the buffer has the re-inserted content.
        assert_eq!(buffer.to_string(), "scribe");
    }
}
