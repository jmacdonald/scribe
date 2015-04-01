use std::ops::Deref;
use std::rc::Rc;
use std::cell::RefCell;
use super::Position;
use super::GapBuffer;

/// Read-only wrapper for a `Position`, to allow field level access to a
/// buffer's cursor while simultaneously enforcing bounds-checking when
/// updating its value.
#[derive(Clone)]
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
        // Don't bother if we are already at the top.
        if self.line == 0 { return; }

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
        let target_line = self.line+1;
        let new_position = Position{ line: target_line, offset: self.offset };

        // Try moving to the same offset on the line below, falling back to its EOL.
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

    /// Decrements the cursor offset. The location is bounds-checked against
    /// the data and the cursor will not be updated if it is out-of-bounds.
    pub fn move_left(&mut self) {
        // Don't bother if we are already at the left edge.
        if self.offset == 0 { return; }

        let new_position = Position{ line: self.line, offset: self.offset-1 };
        self.move_to(new_position);
    }

    /// Increments the cursor offset. The location is bounds-checked against
    /// the data and the cursor will not be updated if it is out-of-bounds.
    pub fn move_right(&mut self) {
        let new_position = Position{ line: self.line, offset: self.offset+1 };
        self.move_to(new_position);
    }

    /// Sets the cursor offset to 0: the start of the current line.
    pub fn move_to_start_of_line(&mut self) {
        let new_position = Position{ line: self.line, offset: 0 };
        self.move_to(new_position);
    }

    /// Moves the cursor offset to after the last character on the current line.
    pub fn move_to_end_of_line(&mut self) {
        let data = self.data.borrow().to_string();
        let current_line = data.lines().nth(self.line);
        match current_line {
            Some(line) => {
                let new_position = Position{ line: self.line, offset: line.len() };
                self.move_to(new_position);
            },
            None => (),
        }
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
    fn move_up_goes_to_eol_if_offset_would_be_out_of_range() {
        let buffer = Rc::new(RefCell::new(gap_buffer::new("This is a test.\nAnother line that is longer.".to_string())));
        let position = Position{ line: 1, offset: 20 };
        let mut cursor = Cursor{ data: buffer, position: position };
        cursor.move_up();
        assert_eq!(cursor.line, 0);
        assert_eq!(cursor.offset, 15);
    }

    #[test]
    fn move_down_goes_to_eol_if_offset_would_be_out_of_range() {
        let buffer = Rc::new(RefCell::new(gap_buffer::new("Another line that is longer.\nThis is a test.".to_string())));
        let position = Position{ line: 0, offset: 20 };
        let mut cursor = Cursor{ data: buffer, position: position };
        cursor.move_down();
        assert_eq!(cursor.line, 1);
        assert_eq!(cursor.offset, 15);
    }

    #[test]
    fn move_to_start_of_line_sets_offset_to_zero() {
        let buffer = Rc::new(RefCell::new(gap_buffer::new("This is a test.\nAnother line.".to_string())));
        let position = Position{ line: 1, offset: 5 };
        let mut cursor = Cursor{ data: buffer, position: position };
        cursor.move_to_start_of_line();
        assert_eq!(cursor.line, 1);
        assert_eq!(cursor.offset, 0);
    }

    #[test]
    fn move_to_end_of_line_sets_offset_the_line_length() {
        let buffer = Rc::new(RefCell::new(gap_buffer::new("This is a test.\nAnother line.".to_string())));
        let position = Position{ line: 0, offset: 5 };
        let mut cursor = Cursor{ data: buffer, position: position };
        cursor.move_to_end_of_line();
        assert_eq!(cursor.line, 0);
        assert_eq!(cursor.offset, 15);
    }

    #[test]
    fn move_up_does_nothing_if_at_the_start_of_line() {
        let buffer = Rc::new(RefCell::new(gap_buffer::new("This is a test.".to_string())));
        let position = Position{ line: 0, offset: 0 };
        let mut cursor = Cursor{ data: buffer, position: position };
        cursor.move_up();
        assert_eq!(cursor.line, 0);
        assert_eq!(cursor.offset, 0);
    }

    #[test]
    fn move_left_does_nothing_if_at_the_start_of_line() {
        let buffer = Rc::new(RefCell::new(gap_buffer::new("This is a test.".to_string())));
        let position = Position{ line: 0, offset: 0 };
        let mut cursor = Cursor{ data: buffer, position: position };
        cursor.move_left();
        assert_eq!(cursor.line, 0);
        assert_eq!(cursor.offset, 0);
    }
}
