use super::Position;

pub struct Range {
    pub start: Position,
    pub end:   Position,
}

impl Range {
    pub fn is_valid(&self) -> bool {
        if self.start.line < self.end.line {
            true
        } else if self.start.line == self.end.line && self.start.offset < self.end.offset {
            true
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Range;
    use super::super::Position;

    #[test]
    fn range_is_valid_when_start_is_before_end() {
        let mut start = Position { line: 0, offset: 4 };
        let mut end = Position { line: 1, offset: 1 };
        let mut range = Range { start: start, end: end };

        assert!(range.is_valid());

        start = Position { line: 1, offset: 4 };
        end = Position { line: 1, offset: 5 };
        range = Range { start: start, end: end };

        assert!(range.is_valid());
    }

    #[test]
    fn range_is_invalid_when_start_is_equal_to_end() {
        let start = Position { line: 0, offset: 4 };
        let end = Position { line: 0, offset: 4 };
        let range = Range { start: start, end: end };

        assert!(!range.is_valid());
    }

    #[test]
    fn range_is_invalid_when_start_is_after_end() {
        let mut start = Position { line: 1, offset: 4 };
        let mut end = Position { line: 1, offset: 1 };
        let mut range = Range { start: start, end: end };

        assert!(!range.is_valid());

        start = Position { line: 2, offset: 4 };
        end = Position { line: 1, offset: 5 };
        range = Range { start: start, end: end };

        assert!(!range.is_valid());
    }
}
