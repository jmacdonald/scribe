use super::position::Position;
use super::range::Range;

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
