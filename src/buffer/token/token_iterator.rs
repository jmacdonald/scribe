use buffer::Token;
use syntect::parsing::ScopeStack;

pub struct TokenIterator {
    data: String,
    stack: ScopeStack,
}

impl Iterator for TokenIterator {
    type Item = Token;

    fn next(&mut self) -> Option<Token> {
        None
    }
}
