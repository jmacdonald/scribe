//! This module contains reversible buffer operations,
//! un/redone using the buffer type's undo/redo methods.
pub use self::delete::Delete;
pub use self::insert::Insert;

mod insert;
mod delete;
