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
    operations: Vec<Box<Operation>>,
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
    fn clone_operation(&self) -> Box<Operation> {
        Box::new(OperationGroup{
            operations: self.operations.iter().map(|o| (*o).clone_operation()).collect()
        })
    }
}

impl OperationGroup {
    /// Adds an operation to the group.
    pub fn add(&mut self, operation: Box<Operation>) {
        self.operations.push(operation);
    }
}

pub fn new() -> OperationGroup {
    OperationGroup{ operations: Vec::new() }
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
                self.operation_group = Some(new());
            }
        }
    }

    /// Tells the buffer to stop tracking operations as a single unit, since
    /// start_operation_group was called. Any calls to insert or delete occurring within
    /// these will be undone/applied together when calling undo/redo, respectively.
    pub fn end_operation_group(&mut self) {
        // Push an open operation group on to the history stack, if one exists.
        match self.operation_group.take() {
            Some(group) => self.history.add(Box::new(group)),
            None => (),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::new;
    use buffer::operations::insert;
    use buffer::Position;
    use buffer::operation::Operation;

    #[test]
    fn run_and_reverse_call_themselves_on_all_operations() {
        let mut group = new();
        let mut buffer = ::buffer::new();

        // Push two insert operations into the group.
        let first = Box::new(insert::new("something".to_string(), Position{ line: 0, offset: 0 }));
        let second = Box::new(insert::new(" else".to_string(), Position{ line: 0, offset: 9 }));
        group.add(first);
        group.add(second);

        // Run the operation group.
        group.run(&mut buffer);

        assert_eq!(buffer.data(), "something else");

        // Reverse the operation group.
        group.reverse(&mut buffer);

        assert_eq!(buffer.data(), "");
    }
}
