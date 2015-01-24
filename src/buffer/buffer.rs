use std::io::{File, Open, ReadWrite};
use std::io::IoResult;
use super::GapBuffer;
use super::gap_buffer;
use super::Position;
use super::Range;

pub struct Buffer {
    data: GapBuffer,
    file: Option<File>,
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

    let data = gap_buffer::new(data);

    // Create a new buffer using the loaded data, file, and other defaults.
    Ok(Buffer{ data: data, file: Some(file) })
}

#[cfg(test)]
mod tests {
    use super::from_file;

    #[test]
    fn from_file_loads_file_into_buffer() {
        match from_file(&Path::new("tests/sample/file")) {
            Ok(buffer) => assert_eq!(buffer.data.to_string(), "it works!\n"),
            Err(error) => panic!(error),
        }
    }
}
