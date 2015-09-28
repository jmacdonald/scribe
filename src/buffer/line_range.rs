use buffer::{Position, Range, range};

/// A more concise expression for ranges spanning complete lines.
pub struct LineRange {
    start: usize,
    end:   usize,
}

impl LineRange {
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
    /// use scribe::buffer::{LineRange, Position, Range, range};
    ///
    /// // Builder a line range.
    /// let line_range = LineRange{ start: 10, end: 14 };
    ///
    /// // Ensure that the resulting range is a zero-based equivalent.
    /// assert_eq!(line_range.to_range(), range::new(
    ///     Position{ line: 10, offset: 0 },
    ///     Position{ line: 14, offset:0 }
    /// ));
    /// ```
    pub fn to_range(&self) -> Range {
        range::new(
            Position{ line: self.start, offset: 0 },
            Position{ line: self.end, offset:0 }
        )
    }

pub fn new(start: usize, end: usize) -> LineRange {
    if start < end {
        LineRange{ start: start, end: end }
    } else {
        LineRange{ start: end, end: start }
    }
}

#[cfg(test)]
mod tests {
    use super::new;

    #[test]
    fn new_does_not_swap_values_if_end_does_not_precede_start() {
        let range = new(0, 1);

        assert_eq!(range.start(), 0);
        assert_eq!(range.end(), 1);
    }

    #[test]
    fn new_swaps_start_and_end_when_end_precedes_start() {
        let range = new(1, 0);

        assert_eq!(range.start(), 0);
        assert_eq!(range.end(), 1);
    }
}
