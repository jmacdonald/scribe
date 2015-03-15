use buffer;
use buffer::Buffer;
use std::old_io::IoError;

pub struct Workspace {
    path: Path,
    buffers: Vec<Buffer>,
    current_buffer_index: Option<usize>,
}

impl Workspace {
    pub fn open_file(&mut self, path: Path) -> Option<IoError> {
        match buffer::from_file(path) {
            Ok(b) => self.buffers.push(b),
            Err(e) => return Some(e),
        }

        None
    }
}

pub fn new(path: Path) -> Workspace {
    Workspace{ path: path, buffers: Vec::new(), current_buffer_index: None }
}

#[cfg(test)]
mod tests {
    use super::new;

    #[test]
    fn open_file_adds_a_properly_initialized_buffer() {
        let mut workspace = new(Path::new("tests/sample"));
        let file_path = Path::new("tests/sample/file");
        workspace.open_file(file_path);

        assert_eq!(workspace.buffers.len(), 1);
    }
}
