use buffer::{Position, Range};

/// A more concise expression for ranges spanning complete lines.
pub struct LineRange {
    pub start: usize,
    pub end:   usize,
}

impl LineRange {
    /// Converts the line range to a regular, zero-offset range.
    ///
    /// # Examples
    ///
    /// ```
    /// use scribe::buffer::{LineRange, Position, Range};
    ///
    /// // Builder a line range.
    /// let line_range = LineRange{ start: 10, end: 14 };
    ///
    /// // Ensure that the resulting range is a zero-based equivalent.
    /// assert_eq!(line_range.to_range(), Range{
    ///     start: Position{ line: 10, offset: 0 },
    ///     end: Position{ line: 14, offset:0 },
    /// });
    /// ```
    pub fn to_range(&self) -> Range {
        Range{
            start: Position{ line: self.start, offset: 0 },
            end: Position{ line: self.end, offset:0 },
        }
    }
}
