// Syntax highlighting
extern crate syntect;

// Grapheme cluster iteration
extern crate unicode_segmentation;

pub mod buffer;
mod workspace;

pub use buffer::Buffer;
pub use workspace::Workspace;
