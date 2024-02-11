pub use self::group::OperationGroup;
use crate::buffer::Buffer;

mod delete;
pub mod group;
pub mod history;
mod insert;

/// A reversible buffer operation.
///
/// Operations are an internal way of encapsulating an action on a buffer
/// that can be run and reversed. They're directly tied to scribe's history
/// functionality, which uses the trait's methods to run and reverse these.
///
/// Types that implement this trait are responsible for adding methods to the
/// Buffer type to expose their functionality; these should build, run, and
/// add the operation objects to the buffer history.
pub trait Operation {
    fn run(&mut self, buffer: &mut Buffer);
    fn reverse(&mut self, buffer: &mut Buffer);
    fn clone_operation(&self) -> Box<dyn Operation>;
}
