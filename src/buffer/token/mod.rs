pub mod line_iterator;
pub mod token_iterator;

use syntect::parsing::Scope;

pub struct Token<'a> {
    pub lexeme: &'a str,
    pub scope: Scope,
    pub scope_depth: usize,
}
