use std::io::{File, Open, ReadWrite};
use std::io::IoResult;

struct GapBuffer {
    buffer: String,
    file: Option<File>,
}

pub fn from_file(path: &Path) -> IoResult<GapBuffer> {
    // Try to open and read the file, returning any errors encountered.
    let mut file = match File::open_mode(path, Open, ReadWrite) {
        Ok(f) => f,
        Err(error) => return Err(error),
    };
    let mut buffer = match file.read_to_string() {
        Ok(b) => b,
        Err(error) => return Err(error),
    };

    // Ensure that the buffer has enough room to grow without reallocating.
    let buffer_length = buffer.len();
    buffer.reserve(buffer_length * 2);

    // Pack the file and buffer into a new GapBuffer object.
    Ok(GapBuffer{ buffer: buffer, file: Some(file) })
}
