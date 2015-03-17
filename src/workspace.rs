use buffer;
use buffer::Buffer;
use std::old_io::IoError;

pub struct Workspace {
    path: Path,
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
}

pub fn new(path: Path) -> Workspace {
    Workspace{ path: path, buffers: Vec::new(), current_buffer_index: None }
}

#[cfg(test)]
mod tests {
    use super::new;
    use buffer;

    #[test]
    fn add_buffer_adds_and_selects_the_passed_buffer() {
        let mut workspace = new(Path::new("tests/sample"));
        let buf = buffer::from_file(Path::new("tests/sample/file")).unwrap();
        workspace.add_buffer(buf);

        assert_eq!(workspace.buffers.len(), 1);
        assert_eq!(workspace.current_buffer().unwrap().data(), "it works!\n");
    }

    #[test]
    fn current_buffer_returns_none_when_there_are_no_buffers() {
        let mut workspace = new(Path::new("tests/sample"));
        assert!(workspace.current_buffer().is_none());
    }

    #[test]
    fn current_buffer_returns_one_when_there_are_buffers() {
        let mut workspace = new(Path::new("tests/sample"));
        let buf = buffer::from_file(Path::new("tests/sample/file")).unwrap();
        workspace.add_buffer(buf);
        assert!(workspace.current_buffer().is_some());
    }

    #[test]
    fn close_current_buffer_does_nothing_when_none_are_open() {
        let mut workspace = new(Path::new("tests/sample"));
        workspace.close_current_buffer();
        assert!(workspace.current_buffer().is_none());
    }

    #[test]
    fn close_current_buffer_cleans_up_when_only_one_buffer_is_open() {
        let mut workspace = new(Path::new("tests/sample"));
        workspace.add_buffer(buffer::new());
        workspace.close_current_buffer();
        assert!(workspace.current_buffer().is_none());
        assert!(workspace.current_buffer_index.is_none());
    }

    #[test]
    fn close_current_buffer_when_two_are_open_selects_the_other() {
        let mut workspace = new(Path::new("tests/sample"));

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
}
