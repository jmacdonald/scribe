pub struct LineIterator<'a> {
    data: &'a str,
    line_number: usize,
    line_start: usize,
    line_end: usize
}

impl<'a> LineIterator<'a> {
    pub fn new<'b>(data: &'b str) -> LineIterator<'b> {
        LineIterator{
            data: data,
            line_number: 0,
            line_start: 0,
            line_end: 0,
        }
    }

    pub fn done(&self) -> bool {
        self.line_end == self.data.len()
    }
}

impl<'a> Iterator for LineIterator<'a> {
    type Item = (usize, &'a str);

    fn next(&mut self) -> Option<Self::Item> {
        if self.done() {
            return None
        }

        // Move the range beyond its previous position.
        self.line_start = self.line_end;

        // Find the next line range.
        for c in self.data[self.line_start..].chars() {
            // Extend the current line range to include this char.
            self.line_end += c.len_utf8();

            if c == '\n' {
                break;
            }
        }
        self.line_number += 1;

        Some((
            self.line_number-1,
            &self.data[
                self.line_start..self.line_end
            ]
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::LineIterator;

    #[test]
    fn next_includes_trailing_newlines() {
        let mut lines = LineIterator::new("line\nanother line\n");
        assert_eq!(Some((0, "line\n")), lines.next());
        assert_eq!(Some((1, "another line\n")), lines.next());
    }

    #[test]
    fn next_stops_at_end_of_data() {
        let mut lines = LineIterator::new("line\nanother line\n");
        lines.next();
        lines.next();
        assert_eq!(None, lines.next());
    }
}
