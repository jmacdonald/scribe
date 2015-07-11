use buffer::{Position, Range};

pub struct LineRange {
    pub start: usize,
    pub end:   usize,
}

impl LineRange {
    pub fn to_range(&self) -> Range {
        Range{
            start: Position{ line: self.start, offset: 0 },
            end: Position{ line: self.end, offset:0 },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::LineRange;
    use buffer::{Position, Range};

    #[test]
    fn to_range_converts_correctly() {
        let line_range = LineRange{ start: 10, end: 14 };

        let expected_range = Range{
            start: Position{ line: 10, offset: 0 },
            end: Position{ line: 14, offset:0 },
        };
        assert_eq!(line_range.to_range(), expected_range);
    }
}
