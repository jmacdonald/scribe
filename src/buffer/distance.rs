/// A vector value representing a span in a buffer. Unlike the
/// Range type, whose two positions are absolutes, a Distance
/// is meant to be used relative to a Position.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Distance {
    pub lines: usize,
    pub offset: usize,
}

impl Distance {
    /// Calculates the distance covered by a string. The
    /// offset is calculated from the last line of the string.
    ///
    /// # Examples
    ///
    /// ```
    /// use scribe::buffer::Distance;
    ///
    /// let data = "scribe\ndistance";
    /// assert_eq!(Distance::of_str(data), Distance{
    ///     lines: 1,
    ///     offset: 8
    /// });
    /// ```
    pub fn of_str(from: &str) -> Distance {
        Distance {
            lines: from.chars().filter(|&c| c == '\n').count(),
            offset: from.split('\n').last().map(|l| l.len()).unwrap_or(0),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Distance;

    #[test]
    fn of_str_works_with_a_single_line_of_data() {
        assert_eq!(
            Distance::of_str("line"),
            Distance {
                lines: 0,
                offset: 4
            }
        );
    }

    #[test]
    fn of_str_works_with_a_trailing_newline() {
        assert_eq!(
            Distance::of_str("trailing newline\n"),
            Distance {
                lines: 1,
                offset: 0
            }
        );
    }
}
