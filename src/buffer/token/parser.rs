use buffer::{Lexeme, Position, Token};
use syntect::parsing::{ParseState, ScopeStack, ScopeStackOp, SyntaxDefinition};
use std::ops::{Deref, DerefMut};

pub struct Parser {
    state: ParseState,
    pub scope: ScopeStack,
    pub position: Position,
}

impl Parser {
    pub fn new(syntax: &SyntaxDefinition) -> Parser {
        Parser {
            state: ParseState::new(syntax),
            scope: ScopeStack::new(),
            position: Position{ line: 0, offset: 0 },
        }
    }
}

impl Deref for Parser {
    type Target = ParseState;

    fn deref(&self) -> &ParseState {
        &self.state
    }
}

impl DerefMut for Parser {
    fn deref_mut<'a>(&'a mut self) -> &'a mut ParseState {
        &mut self.state
    }
}
