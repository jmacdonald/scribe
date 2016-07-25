pub struct LineIterator<'a> {
    data: &'a str,
    start: usize,
    end: usize,
    line_number: usize
}

impl<'a> LineIterator<'a> {
    pub fn new<'b>(data: &'b str) -> LineIterator<'b> {
        LineIterator{
            data: data,
            start: 0,
            end: 0,
            line_number: 0
        }
    }

    pub fn done(&self) -> bool {
        self.end == self.data.len()
    }
}

impl<'a> Iterator for LineIterator<'a> {
    type Item = (usize, &'a str);

    fn next(&mut self) -> Option<Self::Item> {
        if self.done() {
            return None
        }

        // Move the range beyond its previous position.
        self.start = self.end;

        // Find the next line range.
        for (offset, c) in self.data[self.start..].char_indices() {
            // Extend the current line range to include this char.
            self.end += c.len_utf8();

            if c == '\n' {
                break;
            }
        }
        self.line_number += 1;

        Some((self.line_number-1, &self.data[self.start..self.end]))
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
