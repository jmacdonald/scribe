use super::Operation;
use buffer::GapBuffer;

/// A collection of operations that are run as a single/atomic operation. Useful for
/// composing related actions as a single event, from a history/undo standpoint.
pub struct OperationGroup {
    operations: Vec<Box<Operation>>,
}

impl Operation for OperationGroup {
    /// Runs all of the group's individual operations, in order.
    fn run(&mut self, buffer: &mut GapBuffer) {
        for operation in &mut self.operations {
            operation.run(buffer);
        }
    }

    /// Reverses all of the group's individual operations, in reverse order.
    fn reverse(&mut self, buffer: &mut GapBuffer) {
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

#[cfg(test)]
mod tests {
    use super::new;
    use buffer::operations::insert;
    use buffer::Position;
    use buffer::operation::Operation;

    #[test]
    fn run_and_reverse_call_themselves_on_all_operations() {
        let mut group = new();
        let mut buffer = ::buffer::gap_buffer::new(String::new());

        // Push two insert operations into the group.
        let first = Box::new(insert::new("something".to_string(), Position{ line: 0, offset: 0 }));
        let second = Box::new(insert::new(" else".to_string(), Position{ line: 0, offset: 9 }));
        group.add(first);
        group.add(second);

        // Run the operation group.
        group.run(&mut buffer);

        assert_eq!(buffer.to_string(), "something else");

        // Reverse the operation group.
        group.reverse(&mut buffer);

        assert_eq!(buffer.to_string(), "");
    }
}
