// Syntax highlighting
extern crate syntect;

// Grapheme cluster iteration
extern crate unicode_segmentation;

// Error definition/handling
#[macro_use]
extern crate error_chain;

pub mod buffer;
mod errors;
pub mod util;
mod workspace;

pub use crate::buffer::Buffer;
pub use crate::errors::*;
pub use crate::workspace::Workspace;
