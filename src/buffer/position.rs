use buffer::Distance;
use std::cmp::{PartialOrd, Ordering};

/// A two (zero-based) coordinate value representing a location in a buffer.
/// The `offset` field is so named to emphasize that positions point to
/// locations before/after characters, not characters themselves, in an effort
/// to avoid fencepost errors.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Position {
    pub line:   usize,
    pub offset: usize,
}

impl PartialOrd for Position {
    fn partial_cmp(&self, other: &Position) -> Option<Ordering> {
        Some(
            if self.line < other.line {
                Ordering::Less
            } else if self.line > other.line {
                Ordering::Greater
            } else {
                if self.offset < other.offset {
                    Ordering::Less
                } else if self.offset > other.offset {
                    Ordering::Greater
                } else {
                    Ordering::Equal
                }
            }
        )
    }
}

impl Position {
    /// Creates a new position with a line/offset of 0.
    ///
    /// # Examples
    ///
    /// ```
    /// use scribe::buffer::Position;
    ///
    /// let mut position = Position::new();
    /// assert_eq!(position, Position{
    ///     line: 0,
    ///     offset: 0
    /// });
    pub fn new() -> Position {
        Position{ line: 0, offset: 0 }
    }

    /// Adds the specified Distance to the position.
    ///
    /// # Examples
    ///
    /// ```
    /// use scribe::buffer::{Distance, Position};
    ///
    /// let mut position = Position{ line: 1, offset: 3 };
    /// let distance = Distance{ lines: 1, offset: 4 };
    /// position.add(&distance);
    ///
    /// assert_eq!(position, Position{
    ///     line: 2,
    ///     offset: 4
    /// });
    /// ```
    pub fn add(&mut self, distance: &Distance) {
        self.line += distance.lines;

        if distance.lines > 0 {
            self.offset = distance.offset;
        } else {
            self.offset += distance.offset;
        }
    }
}

#[cfg(test)]
mod tests {
    use buffer::{Distance, Position};

    #[test]
    fn compare_works_when_lines_differ() {
        // Important to make the earlier position have a greater
        // offset, since that's an easy mistake to make.
        let earlier_position = Position{ line: 2, offset: 20 };
        let later_position = Position{ line: 3, offset: 10};

        assert!(earlier_position < later_position);
    }

    #[test]
    fn compare_works_when_lines_are_equal() {
        let earlier_position = Position{ line: 3, offset: 10 };
        let later_position = Position{ line: 3, offset: 20};

        assert!(earlier_position < later_position);
    }

    #[test]
    fn compare_works_when_lines_and_offsets_are_equal() {
        let earlier_position = Position{ line: 3, offset: 10 };
        let later_position = Position{ line: 3, offset: 10};

        assert!(earlier_position <= later_position);
        assert!(earlier_position >= later_position);

        // This is technically not necessary since we
        // derive the PartialEq trait, which provides
        // the implementation for this.
        assert!(earlier_position == later_position);
    }

    #[test]
    fn add_works_with_zero_line_distance() {
        let mut position = Position{ line: 1, offset: 3 };
        let distance = Distance{ lines: 0, offset: 4 };
        position.add(&distance);

        assert_eq!(position, Position{
            line: 1,
            offset: 7
        });
    }
}
