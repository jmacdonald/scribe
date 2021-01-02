use super::Operation;
use buffer::Buffer;

/// A collection of operations run as a single/atomic operation.
///
/// Useful for composing smaller, related actions into a larger action, from a history/undo
/// standpoint. A common example of this is character-by-character insertions, which can
/// be undone as a whole, or word by word, instead of one character at a time.
///
/// Because this type implements the Operation trait, it can be placed into the history like
/// any other operation. It's a simple grouping type; it relies on its constituent operations
/// to handle all of their undo/redo implementation details. It exposes two methods on the
/// buffer type to signal the start and end of a group.
pub struct OperationGroup {
    operations: Vec<Box<dyn Operation>>,
}

impl Operation for OperationGroup {
    /// Runs all of the group's individual operations, in order.
    fn run(&mut self, buffer: &mut Buffer) {
        for operation in &mut self.operations {
            operation.run(buffer);
        }
    }

    /// Reverses all of the group's individual operations, in reverse order.
    fn reverse(&mut self, buffer: &mut Buffer) {
        for operation in &mut self.operations.iter_mut().rev() {
            operation.reverse(buffer);
        }
    }

    /// Build a new operation group by manually cloning all of the groups individual operations.
    /// We can't derive this because operations are unsized and need some hand holding.
    fn clone_operation(&self) -> Box<dyn Operation> {
        Box::new(OperationGroup{
            operations: self.operations.iter().map(|o| (*o).clone_operation()).collect()
        })
    }
}

impl OperationGroup {
    /// Creates a new empty operation group.
    pub fn new() -> OperationGroup {
        OperationGroup{ operations: Vec::new() }
    }

    /// Adds an operation to the group.
    pub fn add(&mut self, operation: Box<dyn Operation>) {
        self.operations.push(operation);
    }

    /// Whether or not the group contains any operations.
    pub fn is_empty(&self) -> bool {
        self.operations.is_empty()
    }
}

impl Buffer {
    /// Tells the buffer to start tracking operations as a single unit, until
    /// end_operation_group is called. Any calls to insert or delete occurring within
    /// these will be undone/applied together when calling undo/redo, respectively.
    pub fn start_operation_group(&mut self) {
        // Create an operation group, if one doesn't already exist.
        match self.operation_group {
            Some(_) => (),
            None => {
                self.operation_group = Some(OperationGroup::new());
            }
        }
    }

    /// Tells the buffer to stop tracking operations as a single unit, since
    /// start_operation_group was called. Any calls to insert or delete occurring within
    /// these will be undone/applied together when calling undo/redo, respectively.
    pub fn end_operation_group(&mut self) {
        // Push an open operation group on to the history stack, if one exists.
        if let Some(group) = self.operation_group.take() {
            if !group.is_empty() {
                self.history.add(Box::new(group))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::OperationGroup;
    use buffer::operations::Insert;
    use buffer::{Buffer, Position};
    use buffer::operation::Operation;

    #[test]
    fn run_and_reverse_call_themselves_on_all_operations() {
        let mut group = OperationGroup::new();
        let mut buffer = Buffer::new();

        // Push two insert operations into the group.
        let first = Box::new(Insert::new("something".to_string(), Position{ line: 0, offset: 0 }));
        let second = Box::new(Insert::new(" else".to_string(), Position{ line: 0, offset: 9 }));
        group.add(first);
        group.add(second);

        // Run the operation group.
        group.run(&mut buffer);

        assert_eq!(buffer.data(), "something else");

        // Reverse the operation group.
        group.reverse(&mut buffer);

        assert_eq!(buffer.data(), "");
    }

    #[test]
    fn end_operation_group_drops_group_if_empty() {
        let mut buffer = Buffer::new();
        buffer.insert("amp");

        // Create an empty operation group that
        // shouldn't be added to the buffer history.
        buffer.start_operation_group();
        buffer.end_operation_group();

        // Undo the last change, which should be the initial
        // insert, if the empty operation group was ignored.
        buffer.undo();
        assert_eq!(buffer.data(), "");
    }
}
