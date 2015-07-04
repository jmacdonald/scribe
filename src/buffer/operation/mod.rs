use buffer::Buffer;
pub use self::group::OperationGroup;

pub mod group;
pub mod history;

/// A reversible buffer operation.
///
/// Types that implement this trait are responsible for adding methods to the
/// Buffer type that build, run, and add themselves to the buffer history.
pub trait Operation {
    fn run(&mut self, &mut Buffer);
    fn reverse(&mut self, &mut Buffer);
    fn clone_operation(&self) -> Box<Operation>;
}
