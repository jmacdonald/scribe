use buffer::operation::Operation;

/// Tracks a series of operations.
///
/// Represents a linear history that can be traversed backwards and forwards.
/// Adding a new operation to the history will clear any previously reversed
/// operations, which would otherwise have been eligible to be redone.
pub struct History {
    previous: Vec<Box<Operation>>,
    next: Vec<Box<Operation>>,
}

impl History {
    /// Store an operation that has already been run.
    pub fn add(&mut self, operation: Box<Operation>) {
        self.previous.push(operation);
        self.next.clear();
    }

    /// Navigate the history backwards.
    pub fn previous(&mut self) -> Option<Box<Operation>> {
        match self.previous.pop() {
            Some(operation) => {
                // We've found a previous operation. Before we return it, store a
                // clone of it so that it can be re-applied as a redo operation.
                self.next.push(operation.clone_operation());
                Some(operation)
            },
            None => None
        }
    }

    /// Navigate the history forwards.
    pub fn next(&mut self) -> Option<Box<Operation>> {
        match self.next.pop() {
            Some(operation) => {
                // We've found a subsequent operation. Before we return it, store a
                // clone of it so that it can be re-applied as an undo operation, again.
                self.previous.push(operation.clone_operation());
                Some(operation)
            },
            None => None
        }
    }
}

pub fn new() -> History {
    History{ previous: Vec::new(), next: Vec::new() }
}

#[cfg(test)]
mod tests {
    use super::new;
    use buffer::{Position, operations};
    use buffer::operation::Operation;

    #[test]
    fn previous_and_next_return_the_correct_operations() {
        let mut history = new();

        // Set up a buffer with some data.
        let mut buffer = ::buffer::new();

        // Run an insert operation and add it to the history.
        let insert_position = Position{ line: 0, offset: 0 };
        let mut insert_operation = operations::insert::new("scribe".to_string(), insert_position);
        insert_operation.run(&mut buffer);
        history.add(Box::new(insert_operation));

        // Make sure the buffer has the inserted content.
        assert_eq!(buffer.data(), "scribe");

        // Pull and reverse the last history item.
        match history.previous() {
            Some(mut operation) => operation.reverse(&mut buffer),
            None => (),
        };

        // Make sure the buffer had the inserted content removed.
        assert_eq!(buffer.data(), "");

        // Pull and run the next history item.
        match history.next() {
            Some(mut operation) => operation.run(&mut buffer),
            None => (),
        };

        // Make sure the buffer has the re-inserted content.
        assert_eq!(buffer.data(), "scribe");

        // Pull and reverse the last history item, to make sure
        // the next function properly sets up the previous command.
        match history.previous() {
            Some(mut operation) => operation.reverse(&mut buffer),
            None => (),
        };

        // Make sure the buffer had the inserted content removed.
        assert_eq!(buffer.data(), "");
    }

    #[test]
    fn adding_a_new_operation_clears_redo_stack() {
        let mut history = new();

        // Add an insert operation to the history.
        let insert_position = Position{ line: 0, offset: 0 };
        let mut insert_operation = operations::insert::new("scribe".to_string(), insert_position);
        history.add(Box::new(insert_operation));

        // Pull the last history item. This will
        // add the operation to the redo stack.
        assert!(history.previous().is_some());

        // Add another insert operation to the history.
        let mut second_insert_operation = operations::insert::new("scribe".to_string(), insert_position);
        history.add(Box::new(second_insert_operation));

        // Ensure there are no redo items.
        assert!(history.next().is_none());
    }
}
