use buffer::Token;
use syntect::parsing::{ParseState, ScopeStack, SyntaxDefinition};

pub struct TokenIterator {
    data: String,
    stack: ScopeStack,
    parser: ParseState
}

impl TokenIterator {
    pub fn new(data: String, def: &SyntaxDefinition) -> TokenIterator {
        TokenIterator{
            data: data,
            stack: ScopeStack::new(),
            parser: ParseState::new(def)
        }
    }
}

impl Iterator for TokenIterator {
    type Item = Token;

    fn next(&mut self) -> Option<Token> {
        None
    }
}
