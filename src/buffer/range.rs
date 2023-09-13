use crate::buffer::Position;

/// A two-position type, representing a span of characters.
#[derive(Clone, Debug, PartialEq)]
pub struct Range {
    start: Position,
    end:   Position,
}

impl Range {
    /// Creates a new buffer range. Checks and swaps arguments
    /// in the event that the end precedes the start.
    pub fn new(start: Position, end: Position) -> Range {
        // Ensure that the end does not precede the start.
        if start > end {
            Range{ start: end, end: start }
        } else {
            Range{ start, end }
        }
    }

    pub fn start(&self) -> Position {
        self.start
    }

    pub fn end(&self) -> Position {
        self.end
    }

    /// Whether or not the range includes the specified position.
    /// The range is exclusive, such that its ending position is not included.
    ///
    /// # Examples
    ///
    /// ```
    /// use scribe::buffer::{Position, Range};
    ///
    /// // Builder a range.
    /// let range = Range::new(
    ///     Position{ line: 0, offset: 0 },
    ///     Position{ line: 1, offset: 5 }
    /// );
    ///
    /// assert!(range.includes(
    ///     &Position{ line: 1, offset: 0 }
    /// ));
    ///
    /// assert!(range.includes(
    ///     &Position{ line: 1, offset: 4 }
    /// ));
    ///
    /// assert!(!range.includes(
    ///     &Position{ line: 1, offset: 5 }
    /// ));
    /// ```
    pub fn includes(&self, position: &Position) -> bool {
        position >= &self.start() && position < &self.end()
    }
}

#[cfg(test)]
mod tests {
    use crate::buffer::Position;
    use super::Range;

    #[test]
    fn new_does_not_swap_values_if_end_does_not_precede_start() {
        let mut start = Position { line: 0, offset: 4 };
        let mut end = Position { line: 1, offset: 1 };
        let mut range = Range::new(start, end);

        assert_eq!(range.start(), start);
        assert_eq!(range.end(), end);

        start = Position { line: 0, offset: 4 };
        end = Position { line: 0, offset: 4 };
        range = Range::new(start, end);

        assert_eq!(range.start(), start);
        assert_eq!(range.end(), end);
    }

    #[test]
    fn new_swaps_start_and_end_when_end_precedes_start() {
        let start = Position { line: 1, offset: 4 };
        let end = Position { line: 1, offset: 1 };
        let range = Range::new(start, end);

        assert_eq!(range.start(), end);
        assert_eq!(range.end(), start);
    }
}
