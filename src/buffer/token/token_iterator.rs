use buffer::Token;
use syntect::parsing::ScopeStack;

pub struct TokenIterator {
    data: String,
    stack: ScopeStack,
}

impl TokenIterator {
    pub fn new(data: String) -> TokenIterator {
        TokenIterator{ data: data, stack: ScopeStack::new() }
    }
}

impl Iterator for TokenIterator {
    type Item = Token;

    fn next(&mut self) -> Option<Token> {
        None
    }
}
