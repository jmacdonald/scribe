// Syntax highlighting
extern crate syntect;

// Grapheme cluster iteration
extern crate unicode_segmentation;

// Error definition/handling
#[macro_use]
extern crate error_chain;

pub mod buffer;
mod errors;
mod workspace;

pub use errors::*;
pub use buffer::Buffer;
pub use workspace::Workspace;
