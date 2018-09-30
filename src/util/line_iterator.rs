pub struct LineIterator<'a> {
    data: &'a str,
    line_number: usize,
    line_start: usize,
    line_end: usize,
    done: bool
}

impl<'a> LineIterator<'a> {
    pub fn new(data: &str) -> LineIterator {
        LineIterator{
            data,
            line_number: 0,
            line_start: 0,
            line_end: 0,
            done: false
        }
    }

    fn out_of_data(&self) -> bool {
        self.line_end == self.data.len()
    }
}

impl<'a> Iterator for LineIterator<'a> {
    type Item = (usize, &'a str);

    fn next(&mut self) -> Option<Self::Item> {
        if self.done {
            return None
        }

        // Move the range beyond its previous position.
        self.line_start = self.line_end;

        // We track trailing newlines because, if the buffer ends immediately
        // after one, we want to return one last line on the next iteration.
        let mut trailing_newline = false;

        // Find the next line range.
        for c in self.data[self.line_start..].chars() {
            // Extend the current line range to include this char.
            self.line_end += c.len_utf8();

            if c == '\n' {
                trailing_newline = true;
                break;
            }
        }

        let line = Some((
            self.line_number,
            &self.data[
                self.line_start..self.line_end
            ]
        ));

        // Flag the iterator as done as soon as we've exhausted its data,
        // and have given one last line for data with a trailing newline.
        if self.out_of_data() && !trailing_newline {
            self.done = true;
        } else {
            self.line_number += 1;
        }

        line
    }
}

#[cfg(test)]
mod tests {
    use super::LineIterator;

    #[test]
    fn next_produces_a_value_for_empty_data() {
        let mut lines = LineIterator::new("");
        assert_eq!(Some((0, "")), lines.next());
    }

    #[test]
    fn next_includes_trailing_newlines() {
        let mut lines = LineIterator::new("line\nanother line\n");
        assert_eq!(Some((0, "line\n")), lines.next());
        assert_eq!(Some((1, "another line\n")), lines.next());
        assert_eq!(Some((2, "")), lines.next());
    }

    #[test]
    fn next_stops_at_end_of_data() {
        let mut lines = LineIterator::new("line\nanother line");
        lines.next();
        lines.next();
        assert_eq!(None, lines.next());
    }
}
