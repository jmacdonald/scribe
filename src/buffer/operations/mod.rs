use buffer::Buffer;
use buffer::gap_buffer::GapBuffer;

pub mod insert;

pub trait Operation {
    fn run(&mut self, &mut GapBuffer);
    fn reverse(&mut self, &mut GapBuffer);
}
