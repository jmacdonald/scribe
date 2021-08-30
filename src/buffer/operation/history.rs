use buffer::operation::Operation;

/// Tracks a series of operations.
///
/// Represents a linear history that can be traversed backwards and forwards.
/// Adding a new operation to the history will clear any previously reversed
/// operations, which would otherwise have been eligible to be redone.
pub struct History {
    previous: Vec<Box<dyn Operation>>,
    next: Vec<Box<dyn Operation>>,
    marked_position: Option<usize>
}

impl History {
    /// Creates a new empty operation history.
    pub fn new() -> History {
        History{
            previous: Vec::new(),
            next: Vec::new(),
            marked_position: None
        }
    }

    /// Store an operation that has already been run.
    pub fn add(&mut self, operation: Box<dyn Operation>) {
        self.previous.push(operation);
        self.next.clear();

        // Clear marked position if we've replaced a prior operation.
        if let Some(position) = self.marked_position {
            if position >= self.previous.len() {
                self.marked_position = None
            }
        }
    }

    /// Navigate the history backwards.
    pub fn previous(&mut self) -> Option<Box<dyn Operation>> {
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
    pub fn next(&mut self) -> Option<Box<dyn Operation>> {
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

    pub fn mark(&mut self) {
        self.marked_position = Some(self.previous.len())
    }

    pub fn at_mark(&self) -> bool {
        if let Some(position) = self.marked_position {
            self.previous.len() == position
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::History;
    use buffer::{Buffer, Position};
    use buffer::operations::Insert;
    use buffer::operation::Operation;

    #[test]
    fn previous_and_next_return_the_correct_operations() {
        let mut history = History::new();

        // Set up a buffer with some data.
        let mut buffer = Buffer::new();

        // Run an insert operation and add it to the history.
        let insert_position = Position{ line: 0, offset: 0 };
        let mut insert_operation = Insert::new("scribe".to_string(), insert_position);
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
        let mut history = History::new();

        // Add an insert operation to the history.
        let insert_position = Position{ line: 0, offset: 0 };
        let insert_operation = Insert::new("scribe".to_string(), insert_position);
        history.add(Box::new(insert_operation));

        // Pull the last history item. This will
        // add the operation to the redo stack.
        assert!(history.previous().is_some());

        // Add another insert operation to the history.
        let second_insert_operation = Insert::new("scribe".to_string(), insert_position);
        history.add(Box::new(second_insert_operation));

        // Ensure there are no redo items.
        assert!(history.next().is_none());
    }

    #[test]
    fn marking_the_history_works_without_subsequent_method_calls() {
        let mut history = History::new();
        assert!(!history.at_mark());
        history.mark();
        assert!(history.at_mark());
    }

    #[test]
    fn history_is_not_at_mark_after_adding_an_operation() {
        let mut history = History::new();
        history.mark();

        // Add an insert operation to the history.
        let insert_position = Position{ line: 0, offset: 0 };
        let insert_operation = Insert::new("scribe".to_string(), insert_position);
        history.add(Box::new(insert_operation));

        assert!(!history.at_mark());
    }

    #[test]
    fn history_is_at_mark_after_adding_and_reversing_an_operation() {
        let mut history = History::new();
        history.mark();

        // Add an insert operation to the history.
        let insert_position = Position{ line: 0, offset: 0 };
        let insert_operation = Insert::new("scribe".to_string(), insert_position);
        history.add(Box::new(insert_operation));

        // Reverse the operation.
        history.previous();

        assert!(history.at_mark());
    }

    #[test]
    fn history_is_at_mark_after_toggling_an_operation() {
        let mut history = History::new();

        // Add an insert operation to the history.
        let insert_position = Position{ line: 0, offset: 0 };
        let insert_operation = Insert::new("scribe".to_string(), insert_position);
        history.add(Box::new(insert_operation));

        // Mark the history.
        history.mark();

        // Move to before the operation.
        history.previous();

        // Move to after the operation.
        history.next();

        assert!(history.at_mark());
    }

    #[test]
    fn history_is_not_at_mark_after_replacing_an_operation() {
        let mut history = History::new();

        // Add an insert operation to the history.
        let mut insert_position = Position{ line: 0, offset: 0 };
        let mut insert_operation = Insert::new("scribe".to_string(), insert_position);
        history.add(Box::new(insert_operation));

        // Mark the history.
        history.mark();

        // Move to before the operation.
        history.previous();

        // Add a replacement operation.
        insert_position = Position{ line: 0, offset: 0 };
        insert_operation = Insert::new("scribe".to_string(), insert_position);
        history.add(Box::new(insert_operation));

        assert!(!history.at_mark());
    }
}
