mod line_iterator;
mod token_iterator;
mod token_set;

pub use self::line_iterator::LineIterator;
pub use self::token_iterator::TokenIterator;
pub use self::token_set::TokenSet;

use buffer::Position;
use syntect::parsing::Scope;

#[derive(Debug, PartialEq)]
pub struct Token<'a> {
    pub lexeme: &'a str,
    pub scope: Scope,
    pub position: Position,
}
