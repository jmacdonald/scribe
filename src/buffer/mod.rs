extern crate luthor;

pub use self::buffer::new;
pub use self::buffer::from_file;
pub use self::buffer::Buffer;
pub use self::gap_buffer::GapBuffer;
pub use self::position::Position;
pub use self::range::{Range, LineRange};
pub use self::cursor::Cursor;
pub use self::luthor::token::{Token, Category};

mod buffer;
pub mod gap_buffer;
mod position;
mod range;
pub mod cursor;
mod type_detection;
mod operations;
