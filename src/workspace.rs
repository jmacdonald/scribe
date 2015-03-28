use buffer;
use buffer::Buffer;
use std::path::PathBuf;

pub struct Workspace {
    path: PathBuf,
    buffers: Vec<Buffer>,
    current_buffer_index: Option<usize>,
}

impl Workspace {
    pub fn add_buffer(&mut self, buf: Buffer) {
        self.buffers.push(buf);
        self.current_buffer_index = Some(self.buffers.len()-1);
    }
    
    pub fn current_buffer(&mut self) -> Option<&mut Buffer> {
        match self.current_buffer_index {
            Some(index) => Some(&mut self.buffers[index]),
            None => None,
        }
    }
    
    pub fn close_current_buffer(&mut self) {
        match self.current_buffer_index {
            Some(index) => {
                self.buffers.remove(index);

                if self.buffers.is_empty() {
                    self.current_buffer_index = None;
                } else {
                    self.current_buffer_index = Some(self.buffers.len()-1);
                }
            }
            None => return,
        };
    }

    pub fn previous_buffer(&mut self) {
        match self.current_buffer_index {
            Some(index) => {
                if index > 0 {
                    self.current_buffer_index = Some(index-1);
                } else {
                    self.current_buffer_index = Some(self.buffers.len()-1);
                }
            },
            None => return,
        }
    }

    pub fn next_buffer(&mut self) {
        match self.current_buffer_index {
            Some(index) => {
                if index == self.buffers.len()-1 {
                    self.current_buffer_index = Some(0);
                } else {
                    self.current_buffer_index = Some(index+1);
                }
            },
            None => return,
        }
    }
}

pub fn new(path: PathBuf) -> Workspace {
    Workspace{ path: path, buffers: Vec::new(), current_buffer_index: None }
}

#[cfg(test)]
mod tests {
    use super::new;
    use buffer;
    use std::path::PathBuf;

    #[test]
    fn add_buffer_adds_and_selects_the_passed_buffer() {
        let mut workspace = new(PathBuf::new("tests/sample"));
        let buf = buffer::from_file(PathBuf::new("tests/sample/file")).unwrap();
        workspace.add_buffer(buf);

        assert_eq!(workspace.buffers.len(), 1);
        assert_eq!(workspace.current_buffer().unwrap().data(), "it works!\n");
    }

    #[test]
    fn current_buffer_returns_none_when_there_are_no_buffers() {
        let mut workspace = new(PathBuf::new("tests/sample"));
        assert!(workspace.current_buffer().is_none());
    }

    #[test]
    fn current_buffer_returns_one_when_there_are_buffers() {
        let mut workspace = new(PathBuf::new("tests/sample"));
        let buf = buffer::from_file(PathBuf::new("tests/sample/file")).unwrap();
        workspace.add_buffer(buf);
        assert!(workspace.current_buffer().is_some());
    }

    #[test]
    fn close_current_buffer_does_nothing_when_none_are_open() {
        let mut workspace = new(PathBuf::new("tests/sample"));
        workspace.close_current_buffer();
        assert!(workspace.current_buffer().is_none());
    }

    #[test]
    fn close_current_buffer_cleans_up_when_only_one_buffer_is_open() {
        let mut workspace = new(PathBuf::new("tests/sample"));
        workspace.add_buffer(buffer::new());
        workspace.close_current_buffer();
        assert!(workspace.current_buffer().is_none());
        assert!(workspace.current_buffer_index.is_none());
    }

    #[test]
    fn close_current_buffer_when_two_are_open_selects_the_other() {
        let mut workspace = new(PathBuf::new("tests/sample"));

        // Create two buffers and add them to the workspace.
        let mut first_buffer = buffer::new();
        let mut second_buffer = buffer::new();
        first_buffer.insert("first buffer");
        second_buffer.insert("second buffer");
        workspace.add_buffer(first_buffer);
        workspace.add_buffer(second_buffer);

        // Ensure that the second buffer is currently selected.
        assert_eq!(workspace.current_buffer().unwrap().data(), "second buffer");

        workspace.close_current_buffer();
        assert_eq!(workspace.current_buffer().unwrap().data(), "first buffer");
    }

    #[test]
    fn previous_buffer_does_nothing_when_no_buffers_are_open() {
        let mut workspace = new(PathBuf::new("tests/sample"));
        workspace.previous_buffer();
        assert!(workspace.current_buffer().is_none());
    }

    #[test]
    fn previous_buffer_when_three_are_open_selects_previous_wrapping_to_last() {
        let mut workspace = new(PathBuf::new("tests/sample"));

        // Create two buffers and add them to the workspace.
        let mut first_buffer = buffer::new();
        let mut second_buffer = buffer::new();
        let mut third_buffer = buffer::new();
        first_buffer.insert("first buffer");
        second_buffer.insert("second buffer");
        third_buffer.insert("third buffer");
        workspace.add_buffer(first_buffer);
        workspace.add_buffer(second_buffer);
        workspace.add_buffer(third_buffer);

        // Ensure that the third buffer is currently selected.
        assert_eq!(workspace.current_buffer().unwrap().data(), "third buffer");

        // Ensure that the second buffer is returned.
        workspace.previous_buffer();
        assert_eq!(workspace.current_buffer().unwrap().data(), "second buffer");

        // Ensure that the first buffer is returned.
        workspace.previous_buffer();
        assert_eq!(workspace.current_buffer().unwrap().data(), "first buffer");

        // Ensure that it wraps back to the third buffer.
        workspace.previous_buffer();
        assert_eq!(workspace.current_buffer().unwrap().data(), "third buffer");
    }

    #[test]
    fn next_buffer_does_nothing_when_no_buffers_are_open() {
        let mut workspace = new(PathBuf::new("tests/sample"));
        workspace.next_buffer();
        assert!(workspace.current_buffer().is_none());
    }

    #[test]
    fn next_buffer_when_three_are_open_selects_next_wrapping_to_first() {
        let mut workspace = new(PathBuf::new("tests/sample"));

        // Create two buffers and add them to the workspace.
        let mut first_buffer = buffer::new();
        let mut second_buffer = buffer::new();
        let mut third_buffer = buffer::new();
        first_buffer.insert("first buffer");
        second_buffer.insert("second buffer");
        third_buffer.insert("third buffer");
        workspace.add_buffer(first_buffer);
        workspace.add_buffer(second_buffer);
        workspace.add_buffer(third_buffer);

        // Ensure that the third buffer is currently selected.
        assert_eq!(workspace.current_buffer().unwrap().data(), "third buffer");

        // Ensure that it wraps back to the first buffer.
        workspace.next_buffer();
        assert_eq!(workspace.current_buffer().unwrap().data(), "first buffer");

        // Ensure that the second buffer is returned.
        workspace.next_buffer();
        assert_eq!(workspace.current_buffer().unwrap().data(), "second buffer");

        // Ensure that the third buffer is returned.
        workspace.next_buffer();
        assert_eq!(workspace.current_buffer().unwrap().data(), "third buffer");
    }
}
