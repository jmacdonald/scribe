use crate::buffer::Distance;
use std::cmp::{PartialOrd, Ordering};
use std::default::Default;
use std::ops::{Add, AddAssign};

/// A two (zero-based) coordinate value representing a location in a buffer.
/// The `offset` field is so named to emphasize that positions point to
/// locations before/after characters, not characters themselves, in an effort
/// to avoid fencepost errors.
#[derive(Copy, Clone, Debug, Default, PartialEq)]
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
            } else if self.offset < other.offset {
                Ordering::Less
            } else if self.offset > other.offset {
                Ordering::Greater
            } else {
                Ordering::Equal
            }
        )
    }
}

impl Add<Distance> for Position {
    type Output = Position;

    fn add(self, distance: Distance) -> Self::Output {
        let offset =
            if distance.lines > 0 {
                distance.offset
            } else {
                self.offset + distance.offset
            };

        Position {
            line: self.line + distance.lines,
            offset
        }
    }
}

impl AddAssign<Distance> for Position {
    fn add_assign(&mut self, distance: Distance) {
        self.line += distance.lines;
        self.offset =
            if distance.lines > 0 {
                distance.offset
            } else {
                self.offset + distance.offset
            };
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
        Default::default()
    }
}

#[cfg(test)]
mod tests {
    use crate::buffer::{Distance, Position};

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
    fn add_assign_works_with_zero_line_distance() {
        let mut position = Position{ line: 1, offset: 3 };
        let distance = Distance{ lines: 0, offset: 4 };
        position += distance;

        assert_eq!(position, Position{
            line: 1,
            offset: 7
        });
    }
}
