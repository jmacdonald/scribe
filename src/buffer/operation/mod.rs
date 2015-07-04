use buffer::Buffer;
pub use self::group::OperationGroup;

pub mod group;
pub mod history;

pub trait Operation {
    fn run(&mut self, &mut Buffer);
    fn reverse(&mut self, &mut Buffer);
    fn clone_operation(&self) -> Box<Operation>;
}
