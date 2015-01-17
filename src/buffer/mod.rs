use std::io::{File, Open, ReadWrite};
use std::io::IoResult;

#[cfg(test)]
mod tests;

struct Buffer {
    data: String,
    file: Option<File>,
    cursor: Position,
    selection: Option<Range>,
}

struct Position {
    line:   u64,
    offset: u64,
}

struct Range {
    start: Position,
    end:   Position,
}

impl Range {
    fn is_valid(&self) -> bool {
        if self.start.line < self.end.line {
            true
        } else if self.start.line == self.end.line && self.start.offset < self.end.offset {
            true
        } else {
            false
        }
    }
}

pub fn from_file(path: &Path) -> IoResult<Buffer> {
    // Try to open and read the file, returning any errors encountered.
    let mut file = match File::open_mode(path, Open, ReadWrite) {
        Ok(f) => f,
        Err(error) => return Err(error),
    };
    let mut data = match file.read_to_string() {
        Ok(d) => d,
        Err(error) => return Err(error),
    };

    // Ensure that the data has enough room to grow without reallocating.
    let data_length = data.len();
    data.reserve(data_length * 2);

    // Create a new buffer using the loaded data, file, and other defaults.
    Ok(Buffer{ data: data, file: Some(file), cursor: Position{ line: 0, offset: 0 }, selection: None })
}

#[test]
fn from_file_loads_file_into_buffer() {
    match from_file(&Path::new("tests/sample/file")) {
        Ok(buffer) => assert_eq!(buffer.data, "it works!\n"),
        Err(error) => panic!(error),
    }
}
