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
    
    pub fn current_buffer(&self) -> Option<&Buffer> {
        match self.current_buffer_index {
            Some(index) => Some(&self.buffers[index]),
            None => None,
        }
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
}
