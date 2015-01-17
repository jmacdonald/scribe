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
