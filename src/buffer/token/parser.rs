use buffer::{Lexeme, Position, Token};
use syntect::parsing::{ParseState, ScopeStack, ScopeStackOp, SyntaxDefinition};
use std::ops::{Deref, DerefMut};

pub struct Parser<'a> {
    state: ParseState,
    pub scope: ScopeStack,
    pub position: Position,
}

impl<'a> Parser<'a> {
    pub fn new(syntax: &'a SyntaxDefinition) -> Parser<'a> {
        Parser {
            state: ParseState::new(syntax),
            scope: ScopeStack::new(),
            position: Position{ line: 0, offset: 0 },
        }
    }
}

impl<'a> Deref for Parser<'a> {
    type Target = ParseState;

    fn deref(&self) -> &ParseState {
        &self.state
    }
}

impl<'a> DerefMut for Parser<'a> {
    fn deref_mut(&'a mut self) -> &'a mut ParseState {
        &mut self.state
    }
}
