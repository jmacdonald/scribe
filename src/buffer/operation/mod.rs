use buffer::Buffer;
use buffer::gap_buffer::GapBuffer;
pub use self::group::OperationGroup;

pub mod group;

pub trait Operation {
    fn run(&mut self, &mut GapBuffer);
    fn reverse(&mut self, &mut GapBuffer);
    fn clone(&self) -> Box<Operation>;
}
