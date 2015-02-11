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
    pub fn move_to(&mut self, position: Position) {
        if self.data.borrow().in_bounds(&position) {
            self.position = position;
        }
    }
}
