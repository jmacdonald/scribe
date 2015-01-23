use super::Position;
use super::Range;
use std::str::from_utf8;

pub struct GapBuffer {
    data: Vec<u8>,
    gap_start: usize,
    gap_length: usize,
}

pub fn new(mut data: String) -> GapBuffer {
    let mut bytes = data.into_bytes();
    let capacity = bytes.capacity();
    let gap_start = bytes.len();
    let gap_length = capacity - gap_start;
    unsafe {
        bytes.set_len(capacity);
    }
    GapBuffer{ data: bytes, gap_start: gap_start, gap_length: gap_length }
}

impl GapBuffer {
    // TODO: Return an optional error.
    pub fn insert(&mut self, data: &str, position: &Position) {
        // Ensure we have the capacity to insert this data.
        if data.len() > self.gap_length {
            // TODO: Move gap to the end before resizing buffer.
            self.data.reserve(data.len());
            let capacity = self.data.capacity();
            self.gap_length = capacity - self.gap_start;
            unsafe {
                self.data.set_len(capacity);
            }
        }

        let offset = match self.find_offset(position) {
            Some(o) => o,
            None => return,
        };

        self.move_gap(offset);
        self.write_to_gap(data);
    }

    pub fn to_string(&self) -> String {
        from_utf8(&self.data[..self.gap_start]).unwrap().to_string() + from_utf8(&self.data[self.gap_start+self.gap_length..]).unwrap()
    }

    // Maps a position to its offset equivalent in the data.
    fn find_offset(&self, position: &Position) -> Option<usize> {
        let first_half = from_utf8(&self.data[..self.gap_start]).unwrap();
        let mut line = 0;
        let mut line_offset = 0;

        for char_index in first_half.char_indices() {
            let (offset, character) = char_index;

            // Check to see if we've found the position yet.
            if line == position.line && line_offset == position.offset {
                return Some(offset);
            }

            // Advance the line and offset characters.
            if character == '\n' {
                line+=1;
                line_offset = 0;
            } else {
                line_offset+=1;
            }
        }

        // We didn't find the position *within* the first half, but it could
        // be right after it, which means it's right at the start of the gap.
        if line == position.line && line_offset == position.offset {
            return Some(self.gap_start);
        }

        // We haven't reached the position yet, so we'll move on to the other half.
        let second_half = from_utf8(&self.data[self.gap_start+self.gap_length..]).unwrap();
        for char_index in second_half.char_indices() {
            let (offset, character) = char_index;

            // Check to see if we've found the position yet.
            if line == position.line && line_offset == position.offset {
                return Some(self.gap_start + self.gap_length + offset);
            }

            // Advance the line and offset characters.
            if character == '\n' {
                line+=1;
                line_offset = 0;
            } else {
                line_offset+=1;
            }
        }

        None
    }

    fn move_gap(&mut self, offset: usize) {
        if offset < self.gap_start {
            for index in (offset..self.gap_start) {
                self.data[index + self.gap_length] = self.data[index];
                self.data[index] = 0;
            }
        } else if offset > self.gap_start {
            for index in (self.gap_start+self.gap_length..offset) {
                self.data[index-self.gap_length] = self.data[index];
                self.data[index] = 0;
            }
        }

        self.gap_start = offset;
    }

    fn write_to_gap(&mut self, data: &str) {
        for byte in data.bytes() {
            self.data[self.gap_start] = byte;
            self.gap_start+=1;
            self.gap_length-=1;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::Position;

    #[test]
    fn insert_works() {
        let mut gb = new("This is a test.\nPlease be gentle.".to_string());
        gb.insert(" very", &Position { line: 1, offset: 9 });
        assert_eq!(gb.to_string(), "This is a test.\nPlease be very gentle.");
    }

    #[test]
    fn inserting_at_the_end_works() {
        let mut gb = new("This is a test.".to_string());
        gb.insert(" Seriously.", &Position { line: 0, offset: 15 });
        assert_eq!(gb.to_string(), "This is a test. Seriously.");
    }
}
