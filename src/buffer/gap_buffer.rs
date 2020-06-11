//! Buffer type's underlying data structure.

use super::Position;
use super::Range;
use std::borrow::Borrow;
use unicode_segmentation::UnicodeSegmentation;

/// A UTF-8 string buffer designed to minimize reallocations,
/// maintaining performance amid frequent modifications.
pub struct GapBuffer {
    data: Vec<u8>,
    gap_start: usize,
    gap_length: usize,
}

impl GapBuffer {
    /// Initializes a gap buffer with the specified data as its contents.
    ///
    /// # Examples
    ///
    /// ```
    /// use scribe::buffer::GapBuffer;
    ///
    /// let buffer = GapBuffer::new("scribe".to_string());
    /// assert_eq!(buffer.to_string(), "scribe");
    /// ```
    pub fn new(data: String) -> GapBuffer {
        let mut bytes = data.into_bytes();
        let capacity = bytes.capacity();
        let gap_start = bytes.len();
        let gap_length = capacity - gap_start;
        unsafe {
            bytes.set_len(capacity);
        }

        GapBuffer{ data: bytes, gap_start, gap_length }
    }

    /// Inserts the specified data into the buffer at the specified position.
    /// The buffer will reallocate if there is insufficient space. If the
    /// position is out of bounds, the buffer contents will remain unchanged.
    ///
    /// # Examples
    ///
    /// ```
    /// use scribe::buffer::GapBuffer;
    ///
    /// let mut buffer = GapBuffer::new("my buffer data".to_string());
    /// buffer.insert(" changed", &scribe::buffer::Position{ line: 0, offset: 2});
    /// assert_eq!("my changed buffer data", buffer.to_string());
    /// ```
    pub fn insert(&mut self, data: &str, position: &Position) {
        // Ensure we have the capacity to insert this data.
        if data.len() > self.gap_length {
            // We're about to add space to the end of the buffer, so move the gap
            // there beforehand so that we're essentially just increasing the
            // gap size, and preventing a split/two-segment gap.
            let offset = self.data.capacity();
            self.move_gap(offset);

            // Re-allocate the gap buffer, increasing its size.
            self.data.reserve(data.len());

            // Update the tracked gap size and tell the vector that
            // we're using all of the new space immediately.
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

    /// Returns the specified range of data from the buffer.
    /// If any part of the range does not exist, a none value will be returned.
    ///
    /// # Examples
    ///
    /// ```
    /// use scribe::buffer::{GapBuffer, Range};
    ///
    /// let buffer = GapBuffer::new("my data".to_string());
    /// let range = Range::new(
    ///   scribe::buffer::Position{ line: 0, offset: 3 },
    ///   scribe::buffer::Position{ line: 0, offset: 7}
    /// );
    ///
    /// assert_eq!(buffer.read(&range).unwrap(), "data");
    /// ```
    pub fn read(&self, range: &Range) -> Option<String> {
        // Map positions to offsets in the buffer.
        let start_offset = match self.find_offset(&range.start()) {
            Some(offset) => offset,
            None => return None,
        };
        let end_offset = match self.find_offset(&range.end()) {
            Some(offset) => offset,
            None => return None,
        };

        let data = if start_offset < self.gap_start && self.gap_start < end_offset {
            // The gap is in the middle of the range being requested.
            // Stitch the surrounding halves together to exclude it.
            let first_half = &self.data[start_offset..self.gap_start];
            let second_half = &self.data[self.gap_start+self.gap_length..=end_offset];

            // Allocate a string for the first half.
            let mut data = String::from_utf8_lossy(first_half).into_owned();

            // Push the second half onto the first.
            data.push_str(String::from_utf8_lossy(second_half).borrow());

            data
        } else {
            // No gap in the way; just return the requested data range.
            String::from_utf8_lossy(&self.data[start_offset..=end_offset]).into_owned()
        };

        Some(data)
    }

    /// Returns a string representation of the buffer data (without gap).
    ///
    /// # Examples
    ///
    /// ```
    /// use scribe::buffer::GapBuffer;
    ///
    /// let mut buffer = GapBuffer::new("my data".to_string());
    /// assert_eq!(buffer.to_string(), "my data");
    /// ```
    pub fn to_string(&self) -> String {
        String::from_utf8_lossy(&self.data[..self.gap_start]).to_string() +
        &*String::from_utf8_lossy(&self.data[self.gap_start+self.gap_length..])
    }

    /// Removes the specified range of data from the buffer.
    ///
    /// # Examples
    ///
    /// ```
    /// use scribe::buffer::{GapBuffer, Range};
    ///
    /// let mut buffer = GapBuffer::new("my data".to_string());
    /// let range = Range::new(
    ///   scribe::buffer::Position{ line: 0, offset: 0 },
    ///   scribe::buffer::Position{ line: 0, offset: 3}
    /// );
    ///
    /// buffer.delete(&range);
    /// assert_eq!(buffer.to_string(), "data");
    /// ```
    pub fn delete(&mut self, range: &Range) {
        let start_offset = match self.find_offset(&range.start()) {
            Some(o) => o,
            None => return,
        };
        self.move_gap(start_offset);

        match self.find_offset(&range.end()) {
            Some(offset) => {
                // Widen the gap to cover the deleted contents.
                self.gap_length = offset - self.gap_start;
            },
            None => {
                // The end of the range doesn't exist; check
                // if it's on the last line in the file.
                let start_of_next_line = Position{ line: range.end().line + 1, offset: 0 };

                match self.find_offset(&start_of_next_line) {
                    Some(offset) => {
                        // There are other lines below this range.
                        // Just remove up until the end of the line.
                        self.gap_length = offset - self.gap_start;
                    },
                    None => {
                        // We're on the last line, just get rid of the rest
                        // by extending the gap right to the end of the buffer.
                        self.gap_length = self.data.len() - self.gap_start;
                    }
                }
            }
        };
    }

    /// Checks whether or not the specified position is in bounds of the buffer data.
    ///
    /// # Examples
    ///
    /// ```
    /// use scribe::buffer::GapBuffer;
    ///
    /// let buffer = GapBuffer::new("scribe".to_string());
    /// let in_bounds = scribe::buffer::Position{ line: 0, offset: 0 };
    /// let out_of_bounds = scribe::buffer::Position{ line: 1, offset: 3 };
    ///
    /// assert_eq!(buffer.in_bounds(&in_bounds), true);
    /// assert_eq!(buffer.in_bounds(&out_of_bounds), false);
    /// ```
    pub fn in_bounds(&self, position: &Position) -> bool {
        self.find_offset(position) != None
    }

    // Maps a position to its offset equivalent in the data.
    fn find_offset(&self, position: &Position) -> Option<usize> {
        let first_half = String::from_utf8_lossy(&self.data[..self.gap_start]);
        let mut line = 0;
        let mut line_offset = 0;

        for (offset, grapheme) in (&*first_half).grapheme_indices(true) {
            // Check to see if we've found the position yet.
            if line == position.line && line_offset == position.offset {
                return Some(offset);
            }

            // Advance the line and offset characters.
            if grapheme == "\n" {
                line+=1;
                line_offset = 0;
            } else {
                line_offset+=1;
            }
        }

        // We didn't find the position *within* the first half, but it could
        // be right after it, which means it's right at the start of the gap.
        if line == position.line && line_offset == position.offset {
            return Some(self.gap_start+self.gap_length);
        }

        // We haven't reached the position yet, so we'll move on to the other half.
        let second_half = String::from_utf8_lossy(&self.data[self.gap_start+self.gap_length..]);
        for (offset, grapheme) in (&*second_half).grapheme_indices(true) {
            // Check to see if we've found the position yet.
            if line == position.line && line_offset == position.offset {
                return Some(self.gap_start + self.gap_length + offset);
            }

            // Advance the line and offset characters.
            if grapheme == "\n" {
                line+=1;
                line_offset = 0;
            } else {
                line_offset+=1;
            }
        }

        // We didn't find the position *within* the second half, but it could
        // be right after it, which means it's at the end of the buffer.
        if line == position.line && line_offset == position.offset {
            return Some(self.data.len());
        }

        None
    }

    fn move_gap(&mut self, offset: usize) {
        // We don't need to move any data if the buffer is at capacity.
        if self.gap_length == 0 {
            self.gap_start = offset;
            return;
        }

        if offset < self.gap_start {
            // Shift the gap to the left one byte at a time.
            for index in (offset..self.gap_start).rev() {
                self.data[index + self.gap_length] = self.data[index];
                self.data[index] = 0;
            }

            self.gap_start = offset;
        } else if offset > self.gap_start {
            // Shift the gap to the right one byte at a time.
            for index in self.gap_start + self.gap_length..offset {
                self.data[index-self.gap_length] = self.data[index];
                self.data[index] = 0;
            }

            // Because the offset was after the gap, its value included the
            // gap length. We must remove it to determine the starting point.
            self.gap_start = offset - self.gap_length;
        }
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
    use buffer::{GapBuffer, Position, Range};

    #[test]
    fn move_gap_works() {
        let mut gb = GapBuffer::new("This is a test.".to_string());
        gb.move_gap(0);
        assert_eq!(gb.to_string(), "This is a test.");
    }

    #[test]
    fn inserting_at_the_start_works() {
        let mut gb = GapBuffer::new("toolkit".to_string());

        // This insert serves to move the gap to the start of the buffer.
        gb.insert(" ", &Position { line: 0, offset: 0 });
        assert_eq!(gb.to_string(), " toolkit");

        // We insert enough data (more than twice original capacity) to force
        // a re-allocation, which, with the gap at the start and when handled
        // incorrectly, will create a split/two-segment gap. Bad news.
        gb.insert("scribe text", &Position { line: 0, offset: 0 });
        assert_eq!(gb.to_string(), "scribe text toolkit");
    }

    #[test]
    fn inserting_in_the_middle_works() {
        let mut gb = GapBuffer::new("    editor".to_string());

        // Same deal as above "at the start" test, where we want to move
        // the gap into the middle and then force a reallocation to check
        // that pre-allocation gap shifting is working correctly.
        gb.insert(" ", &Position { line: 0, offset: 4 });
        gb.insert("scribe", &Position { line: 0, offset: 4 });
        assert_eq!(gb.to_string(), "    scribe editor");
    }

    #[test]
    fn inserting_at_the_end_works() {
        let mut gb = GapBuffer::new("This is a test.".to_string());
        gb.insert(" Seriously.", &Position { line: 0, offset: 15 });
        assert_eq!(gb.to_string(), "This is a test. Seriously.");
    }

    #[test]
    fn inserting_in_different_spots_twice_works() {
        let mut gb = GapBuffer::new("This is a test.".to_string());
        gb.insert("Hi. ", &Position { line: 0, offset: 0 });
        gb.insert(" Thank you.", &Position { line: 0, offset: 19 });
        assert_eq!(gb.to_string(), "Hi. This is a test. Thank you.");
    }

    #[test]
    fn inserting_at_an_invalid_position_does_nothing() {
        let mut gb = GapBuffer::new("This is a test.".to_string());
        gb.insert(" Seriously.", &Position { line: 0, offset: 35 });
        assert_eq!(gb.to_string(), "This is a test.");
    }

    #[test]
    fn inserting_after_a_grapheme_cluster_works() {
        let mut gb = GapBuffer::new("scribe नी".to_string());
        gb.insert(" library", &Position{ line : 0, offset: 8 });
        assert_eq!(gb.to_string(), "scribe नी library");
    }

    #[test]
    fn deleting_works() {
        let mut gb = GapBuffer::new("This is a test.\nSee what happens.".to_string());
        let start = Position{ line: 0, offset: 8 };
        let end = Position{ line: 1, offset: 4 };
        gb.delete(&Range::new(start, end));
        assert_eq!(gb.to_string(), "This is what happens.");
    }

    #[test]
    fn inserting_then_deleting_at_the_start_works() {
        let mut gb = GapBuffer::new(String::new());
        gb.insert("This is a test.", &Position{ line: 0, offset: 0});
        let start = Position{ line: 0, offset: 0 };
        let end = Position{ line: 0, offset: 1 };
        gb.delete(&Range::new(start, end));
        assert_eq!(gb.to_string(), "his is a test.");
    }

    #[test]
    fn deleting_immediately_after_the_end_of_the_gap_widens_the_gap() {
        let mut gb = GapBuffer::new("This is a test.".to_string());
        let mut start = Position{ line: 0, offset: 8 };
        let mut end = Position{ line: 0, offset: 9 };
        gb.delete(&Range::new(start, end));
        assert_eq!(gb.to_string(), "This is  test.");
        assert_eq!(gb.gap_length, 1);

        start = Position{ line: 0, offset: 9 };
        end = Position{ line: 0, offset: 10 };
        gb.delete(&Range::new(start, end));
        assert_eq!(gb.to_string(), "This is  est.");
        assert_eq!(gb.gap_length, 2);
    }

    #[test]
    fn deleting_to_an_out_of_range_line_deletes_to_the_end_of_the_buffer() {
        let mut gb = GapBuffer::new("scribe\nlibrary".to_string());
        let start = Position{ line: 0, offset: 6 };
        let end = Position{ line: 2, offset: 10 };
        gb.delete(&Range::new(start, end));
        assert_eq!(gb.to_string(), "scribe");
    }

    #[test]
    fn deleting_to_an_out_of_range_column_deletes_to_the_end_of_the_buffer() {
        let mut gb = GapBuffer::new("scribe\nlibrary".to_string());
        let start = Position{ line: 0, offset: 0 };
        let end = Position{ line: 0, offset: 100 };
        gb.delete(&Range::new(start, end));
        assert_eq!(gb.to_string(), "library");
    }

    #[test]
    fn deleting_after_a_grapheme_cluster_works() {
        let mut gb = GapBuffer::new("scribe नी library".to_string());
        let start = Position{ line: 0, offset: 8 };
        let end = Position{ line: 0, offset: 16 };
        gb.delete(&Range::new(start, end));
        assert_eq!(gb.to_string(), "scribe नी");
    }

    #[test]
    fn read_does_not_include_gap_contents_when_gap_is_at_start_of_range() {
        // Create a buffer and a range that captures the first character.
        let mut gb = GapBuffer::new("scribe".to_string());
        let range = Range::new(
            Position{ line: 0, offset: 0 },
            Position{ line: 0, offset: 1 }
        );

        // Delete the first character, which will move the gap buffer to the start.
        gb.delete(&range);
        assert_eq!(gb.to_string(), "cribe");

        // Ask for the first character, which would include the deleted
        // value, if the read function isn't smart enough to skip it.
        assert_eq!(gb.read(&range).unwrap(), "c");
    }

    #[test]
    fn read_does_not_include_gap_contents_when_gap_is_in_middle_of_range() {
        let mut gb = GapBuffer::new("scribe".to_string());

        // Delete data from the middle of the buffer, which will move the gap there.
        gb.delete(&Range::new(
            Position{ line: 0, offset: 2 },
            Position{ line: 0, offset: 4 }
        ));
        assert_eq!(gb.to_string(), "scbe");

        // Request a range that extends from the start to the finish.
        let range = Range::new(
            Position{ line: 0, offset: 0 },
            Position{ line: 0, offset: 4 }
        );
        assert_eq!(gb.read(&range).unwrap(), "scbe");
    }

    #[test]
    fn reading_after_a_grapheme_cluster_works() {
        let gb = GapBuffer::new("scribe नी library".to_string());
        let range = Range::new(
            Position{ line: 0, offset: 8 },
            Position{ line: 0, offset: 16 }
        );
        assert_eq!(gb.read(&range).unwrap(), " library");
    }

    #[test]
    fn in_bounds_considers_grapheme_clusters() {
        let gb = GapBuffer::new("scribe नी library".to_string());
        let in_bounds = Position{ line: 0, offset: 16 };
        let out_of_bounds = Position{ line: 0, offset: 17 };
        assert!(gb.in_bounds(&in_bounds));
        assert!(!gb.in_bounds(&out_of_bounds));
    }
}
