use buffer::Position;

/// A two-position type, representing a span of characters.
#[derive(Clone, Debug, PartialEq)]
pub struct Range {
    start: Position,
    end:   Position,
}

impl Range {
    pub fn start(&self) -> Position {
        self.start
    }
    
    pub fn end(&self) -> Position {
        self.end
    }

    pub fn includes(&self, position: &Position) -> bool {
        position >= &self.start() && position < &self.end()
    }
}

pub fn new(start: Position, end: Position) -> Range {
    // Ensure that the end does not precede the start.
    if start > end {
        Range{ start: end, end: start }
    } else {
        Range{ start: start, end: end }
    }
}

#[cfg(test)]
mod tests {
    use buffer::Position;
    use super::new;

    #[test]
    fn new_does_not_swap_values_if_end_does_not_precede_start() {
        let mut start = Position { line: 0, offset: 4 };
        let mut end = Position { line: 1, offset: 1 };
        let mut range = new(start, end);

        assert_eq!(range.start(), start);
        assert_eq!(range.end(), end);

        start = Position { line: 0, offset: 4 };
        end = Position { line: 0, offset: 4 };
        range = new(start, end);

        assert_eq!(range.start(), start);
        assert_eq!(range.end(), end);
    }

    #[test]
    fn new_swaps_start_and_end_when_end_precedes_start() {
        let start = Position { line: 1, offset: 4 };
        let end = Position { line: 1, offset: 1 };
        let range = new(start, end);

        assert_eq!(range.start(), end);
        assert_eq!(range.end(), start);
    }

    #[test]
    fn includes_is_true_when_position_is_in_range() {
        let start = Position { line: 0, offset: 0 };
        let end = Position { line: 2, offset: 0 };
        let range = new(start, end);

        assert!(range.includes(&Position{
            line: 1,
            offset: 0
        }));
    }

    #[test]
    fn includes_is_true_when_position_is_on_end_line_but_within_range() {
        let start = Position { line: 0, offset: 0 };
        let end = Position { line: 1, offset: 5 };
        let range = new(start, end);

        assert!(range.includes(&Position{
            line: 1,
            offset: 3 
        }));
    }

    #[test]
    fn includes_is_false_when_position_is_out_of_range() {
        let start = Position { line: 0, offset: 0 };
        let end = Position { line: 2, offset: 0 };
        let range = new(start, end);

        assert!(!range.includes(&Position{
            line: 3,
            offset: 0
        }));
    }
}
