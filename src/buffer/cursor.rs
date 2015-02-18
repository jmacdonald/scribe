use std::ops::Deref;
use std::rc::Rc;
use std::cell::RefCell;
use super::Position;
use super::GapBuffer;

/// Read-only wrapper for a `Position`, to allow field level access to a
/// buffer's cursor while simultaneously enforcing bounds-checking when
/// updating its value.
pub struct Cursor {
    pub data: Rc<RefCell<GapBuffer>>,
    pub position: Position,
}

impl Deref for Cursor {
    type Target = Position;

    fn deref(&self) -> &Position {
        &self.position
    }
}

impl Cursor {
    /// Moves the cursor to the specified location. The location is
    /// bounds-checked against the data and the cursor will not be
    /// updated if it is out-of-bounds.
    ///
    /// # Examples
    ///
    /// ```
    /// let mut buffer = scribe::buffer::new();
    /// let in_bounds = scribe::buffer::Position{ line: 0, offset: 2 };
    /// let out_of_bounds = scribe::buffer::Position{ line: 2, offset: 2 };
    /// buffer.insert("scribe");
    ///
    /// buffer.cursor.move_to(in_bounds);
    /// assert_eq!(buffer.cursor.line, 0);
    /// assert_eq!(buffer.cursor.offset, 2);
    ///
    /// buffer.cursor.move_to(out_of_bounds);
    /// assert_eq!(buffer.cursor.line, 0);
    /// assert_eq!(buffer.cursor.offset, 2);
    /// ```
    pub fn move_to(&mut self, position: Position) -> bool {
        if self.data.borrow().in_bounds(&position) {
            self.position = position;
            return true
        }
        false
    }

    /// Decrements the cursor line. The location is bounds-checked against
    /// the data and the cursor will not be updated if it is out-of-bounds.
    pub fn move_up(&mut self) {
        let target_line = self.line-1;
        let new_position = Position{ line: target_line, offset: self.offset };

        // Try moving to the same offset on the line above, falling back to its EOL.
        if self.move_to(new_position) == false {
            let mut target_offset = 0;
            for (line_number, line) in self.data.borrow().to_string().lines().enumerate() {
                if line_number == target_line {
                    target_offset = line.len();
                }
            }
            self.move_to(Position{ line: target_line, offset: target_offset });
        }
    }

    /// Increments the cursor line. The location is bounds-checked against
    /// the data and the cursor will not be updated if it is out-of-bounds.
    pub fn move_down(&mut self) {
        let new_position = Position{ line: self.line+1, offset: self.offset };
        self.move_to(new_position);
    }

    /// Decrements the cursor offset. The location is bounds-checked against
    /// the data and the cursor will not be updated if it is out-of-bounds.
    pub fn move_left(&mut self) {
        let new_position = Position{ line: self.line, offset: self.offset-1 };
        self.move_to(new_position);
    }

    /// Increments the cursor offset. The location is bounds-checked against
    /// the data and the cursor will not be updated if it is out-of-bounds.
    pub fn move_right(&mut self) {
        let new_position = Position{ line: self.line, offset: self.offset+1 };
        self.move_to(new_position);
    }
}

#[cfg(test)]
mod tests {
    use super::Cursor;
    use super::super::gap_buffer;
    use super::super::Position;
    use std::rc::Rc;
    use std::cell::RefCell;

    #[test]
    fn move_up_goes_to_EOL_if_offset_would_be_out_of_range() {
        let mut buffer = Rc::new(RefCell::new(gap_buffer::new("This is a test.\nAnother line that is longer.".to_string())));
        let position = Position{ line: 1, offset: 20 };
        let mut cursor = Cursor{ data: buffer, position: position };
        cursor.move_up();
        assert_eq!(cursor.line, 0);
        assert_eq!(cursor.offset, 15);
    }
}
