use buffer::{Position, Range};

/// A more concise expression for ranges spanning complete lines.
#[derive(PartialEq, Debug)]
pub struct LineRange {
    start: usize,
    end:   usize,
}

impl LineRange {
    /// Creates a new buffer line range. Checks and swaps
    /// arguments in the event that the end precedes the start.
    pub fn new(start: usize, end: usize) -> LineRange {
        if start < end {
            LineRange{ start, end }
        } else {
            LineRange{ start: end, end: start }
        }
    }

    pub fn start(&self) -> usize {
        self.start
    }

    pub fn end(&self) -> usize {
        self.end
    }

    /// Converts the line range to a regular, zero-offset range.
    ///
    /// # Examples
    ///
    /// ```
    /// use scribe::buffer::{LineRange, Position, Range};
    ///
    /// // Build a line range.
    /// let line_range = LineRange::new(10, 14);
    ///
    /// // Ensure that the resulting range is a zero-based equivalent.
    /// assert_eq!(line_range.to_range(), Range::new(
    ///     Position{ line: 10, offset: 0 },
    ///     Position{ line: 14, offset:0 }
    /// ));
    /// ```
    pub fn to_range(&self) -> Range {
        Range::new(
            Position{ line: self.start, offset: 0 },
            Position{ line: self.end, offset:0 }
        )
    }

    /// Converts the line range to a regular, zero-offset range, including
    /// the line on which the range ends.
    ///
    /// # Examples
    ///
    /// ```
    /// use scribe::buffer::{LineRange, Position, Range};
    ///
    /// // Build a line range.
    /// let line_range = LineRange::new(10, 14);
    ///
    /// // Ensure that the resulting range is a zero-based equivalent.
    /// assert_eq!(line_range.to_inclusive_range(), Range::new(
    ///     Position{ line: 10, offset: 0 },
    ///     Position{ line: 15, offset:0 }
    /// ));
    /// ```
    pub fn to_inclusive_range(&self) -> Range {
        Range::new(
            Position{ line: self.start, offset: 0 },
            Position{ line: self.end+1, offset:0 }
        )
    }

    /// Whether or not the line range includes the specified line.
    /// The range is exclusive, such that its ending line is not included.
    ///
    /// # Examples
    ///
    /// ```
    /// use scribe::buffer::LineRange;
    ///
    /// // Build a line range.
    /// let line_range = LineRange::new(10, 14);
    ///
    /// assert!(line_range.includes(11));
    /// assert!(!line_range.includes(14));
    /// ```
    pub fn includes(&self, line: usize) -> bool {
        line >= self.start() && line < self.end()
    }
}

#[cfg(test)]
mod tests {
    use super::LineRange;

    #[test]
    fn new_does_not_swap_values_if_end_does_not_precede_start() {
        let range = LineRange::new(0, 1);

        assert_eq!(range.start(), 0);
        assert_eq!(range.end(), 1);
    }

    #[test]
    fn new_swaps_start_and_end_when_end_precedes_start() {
        let range = LineRange::new(1, 0);

        assert_eq!(range.start(), 0);
        assert_eq!(range.end(), 1);
    }
}
