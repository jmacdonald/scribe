mod gap_buffer;

#[cfg(test)]
mod tests;

struct Position {
    line:   u64,
    offset: u64,
}

struct Range {
    start: Position,
    end:   Position,
}

impl Range {
    fn is_valid(&self) -> bool {
        if self.start.line < self.end.line {
            true
        } else if self.start.line == self.end.line && self.start.offset < self.end.offset {
            true
        } else {
            false
        }
    }
}
