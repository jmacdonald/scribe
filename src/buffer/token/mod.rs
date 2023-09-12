mod token_iterator;
mod token_set;

pub use self::token_iterator::TokenIterator;
pub use self::token_set::TokenSet;

use crate::buffer::Position;
use syntect::parsing::ScopeStack;

#[derive(Debug, PartialEq)]
pub enum Token<'a> {
    Newline,
    Lexeme(Lexeme<'a>)
}

#[derive(Debug, PartialEq)]
pub struct Lexeme<'a> {
    pub value: &'a str,
    pub scope: ScopeStack,
    pub position: Position,
}
