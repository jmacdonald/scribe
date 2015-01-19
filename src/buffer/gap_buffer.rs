use super::Position;
use super::Range;

pub struct GapBuffer {
    data: String,
}

pub fn new(mut data: String) -> GapBuffer {
    // Ensure that the data has enough room to grow without reallocating.
    let data_length = data.len();
    data.reserve(data_length * 2);

    GapBuffer{ data: data }
}

impl GapBuffer {
    pub fn to_string(&self) -> String {
        self.data.clone()
    }
}
